#![no_std]
#![no_main]

use cortex_m_rt::entry;

use cortex_m_semihosting::hprintln;
use panic_halt as _;

// Needed for linking
#[allow(unused_imports)]
use stm32f2::stm32f217 as _;
use stm32f2::stm32f217::Peripherals;
use stm32f207_hal::{
    delay::Delay,
    ethernet::{
        device::EthernetDevice,
        pins::MiiPins,
        ring::{Receive, RingEntry, Transmit},
    },
    interrupt_free_cell::InterruptFreeCell,
    prelude::*,
};

use smoltcp::{
    iface::{Interface, InterfaceBuilder, NeighborCache, Routes, SocketStorage},
    phy::Device,
    socket::{Dhcpv4Event, Dhcpv4Socket, IcmpPacketMetadata, IcmpSocket, IcmpSocketBuffer},
    time::{Duration, Instant},
    wire::{EthernetAddress, IpAddress, IpCidr, Ipv4Address, Ipv4Cidr},
};

#[entry]
fn main() -> ! {
    hprintln!("Starting...").unwrap();

    let peripherals = Peripherals::take().unwrap();
    let core_peripherals = cortex_m::Peripherals::take().unwrap();

    let rcc = peripherals.RCC.constrain();

    let clocks = rcc.cfgr.sysclk(32.mhz()).hclk(32.mhz()).freeze();

    // Setup pins
    let gpio_a = peripherals.GPIOA.split();
    let gpio_b = peripherals.GPIOB.split();
    let gpio_c = peripherals.GPIOC.split();
    let gpio_g = peripherals.GPIOG.split();
    let gpio_h = peripherals.GPIOH.split();
    let gpio_i = peripherals.GPIOI.split();

    let mii_pins = MiiPins {
        transmit_clk: gpio_c.pc3.into_alternate::<11>(),
        receive_clk: gpio_a.pa1.into_alternate::<11>(),
        transmit_en: gpio_g.pg11.into_alternate::<11>(),
        transmit_d0: gpio_g.pg13.into_alternate::<11>(),
        transmit_d1: gpio_g.pg14.into_alternate::<11>(),
        transmit_d2: gpio_c.pc2.into_alternate::<11>(),
        transmit_d3: gpio_b.pb8.into_alternate::<11>(),
        crs: gpio_h.ph2.into_alternate::<11>(),
        col: gpio_h.ph3.into_alternate::<11>(),
        receive_d0: gpio_c.pc4.into_alternate::<11>(),
        receive_d1: gpio_c.pc5.into_alternate::<11>(),
        receive_d2: gpio_h.ph6.into_alternate::<11>(),
        receive_d3: gpio_h.ph7.into_alternate::<11>(),
        receive_dv: gpio_a.pa7.into_alternate::<11>(),
        receive_er: gpio_i.pi10.into_alternate::<11>(),
    };

    // Setup the delay
    let mut delay = Delay::new(core_peripherals.SYST, clocks);

    // Setup the smoltcp interface
    let Buffers {
        receive_buffer,
        transmit_buffer,
        icmp_receive_metadata_buffer,
        icmp_transmit_metadata_buffer,
        icmp_receive_payload_buffer,
        icmp_transmit_payload_buffer,
    } = get_buffers().unwrap();

    let ethernet = EthernetDevice::new(
        peripherals.ETHERNET_MAC,
        peripherals.ETHERNET_DMA,
        peripherals.ETHERNET_PTP,
        receive_buffer.as_mut_slice(),
        transmit_buffer.as_mut_slice(),
        clocks,
        mii_pins,
    )
    .expect("Could not build device");

    let mut sockets: [SocketStorage; 2] = Default::default();

    hprintln!("Setting up the interface ethernet").unwrap();

    let mut neighbor_cache_entries = [None; 8];
    let neighbor_cache = NeighborCache::new(neighbor_cache_entries.as_mut_slice());

    let mut ip_address = [IpCidr::new(IpAddress::Ipv4(Ipv4Address::UNSPECIFIED), 0)];
    let mut routes = [None];
    let mut interface = InterfaceBuilder::new(ethernet, sockets.as_mut_slice())
        .ip_addrs(ip_address.as_mut_slice())
        .routes(Routes::new(routes.as_mut_slice()))
        .hardware_addr(EthernetAddress([0x02, 0x00, 0x00, 0x00, 0x00, 0x02]).into()) // What about [0x02, 0x00, 0x00, 0x00, 0x00, 0x02] ?
        .neighbor_cache(neighbor_cache)
        .finalize();

    let dhcp_socket = Dhcpv4Socket::new();
    let icmp_socket = IcmpSocket::new(
        IcmpSocketBuffer::new(
            icmp_receive_metadata_buffer.as_mut_slice(),
            icmp_receive_payload_buffer.as_mut_slice(),
        ),
        IcmpSocketBuffer::new(
            icmp_transmit_metadata_buffer.as_mut_slice(),
            icmp_transmit_payload_buffer.as_mut_slice(),
        ),
    );

    let dhcp_handle = interface.add_socket(dhcp_socket);
    // direct access to the icmp socket not needed
    // smoltcp already handles pings internally
    let _icmp_handle = interface.add_socket(icmp_socket);

    hprintln!("Listening for messages!").unwrap();

    let mut timestamp = Instant::from_millis(0);

    loop {
        if let Err(e) = interface.poll(timestamp) {
            hprintln!("poll error: {:?}", e).unwrap();
        };

        // Handle DHCP Messages
        handle_dhcp_messages(&mut interface, dhcp_handle);

        // Determine how long to wait before the next poll
        const DURATION_1US: Duration = Duration::from_micros(1);
        let delay_duration = interface.poll_delay(timestamp).unwrap_or(DURATION_1US);

        // wait for the recommended amount of time
        timestamp += delay_duration;
        delay.delay_us(delay_duration.micros() as u32);
    }
}

fn handle_dhcp_messages<'r, 't>(
    interface: &mut Interface<EthernetDevice<'r, 't>>,
    dhcp_handle: smoltcp::iface::SocketHandle,
) where
    for<'d> EthernetDevice<'r, 't>: Device<'d>,
{
    let dhcp_socket = interface.get_socket::<Dhcpv4Socket>(dhcp_handle);
    if let Some(event) = dhcp_socket.poll() {
        match event {
            Dhcpv4Event::Configured(config) => {
                let address = config.address;
                let router = config.router;

                hprintln!("DHCP configured, IP: {}", address,).unwrap();

                set_ipv4_address(interface, address);

                match router {
                    Some(router) => interface
                        .routes_mut()
                        .add_default_ipv4_route(router)
                        .expect("Could net set Default Gateway"),
                    None => interface.routes_mut().remove_default_ipv4_route(),
                };
            }
            Dhcpv4Event::Deconfigured => {
                hprintln!("DHCP Config lost").unwrap();
                set_ipv4_address(interface, Ipv4Cidr::new(Ipv4Address::UNSPECIFIED, 0));
                interface.routes_mut().remove_default_ipv4_route();
            }
        }
    }
}

fn set_ipv4_address<T>(interface: &mut Interface<'_, T>, cidr: Ipv4Cidr)
where
    T: for<'a> Device<'a>,
{
    interface.update_ip_addrs(|addresses| *addresses.first_mut().unwrap() = IpCidr::Ipv4(cidr))
}

struct Buffers {
    receive_buffer: &'static mut [RingEntry<Receive>; 16],
    transmit_buffer: &'static mut [RingEntry<Transmit>; 16],
    icmp_receive_metadata_buffer: &'static mut [IcmpPacketMetadata; 16],
    icmp_transmit_metadata_buffer: &'static mut [IcmpPacketMetadata; 16],
    icmp_receive_payload_buffer: &'static mut [u8; 1024],
    icmp_transmit_payload_buffer: &'static mut [u8; 1024],
}

fn get_buffers() -> Option<Buffers> {
    // This variable is used to check that the buffers have only been given out once
    static BUFFERS_HANDED_OUT: InterruptFreeCell<bool> = InterruptFreeCell::new(false);

    let buffers_handed_out = BUFFERS_HANDED_OUT.replace(true);
    if buffers_handed_out {
        return None;
    }

    // Im not aware how this repetition can be avoid
    static mut RECEIVE_BUFFER: [RingEntry<Receive>; 16] = [
        RingEntry::new_receive(),
        RingEntry::new_receive(),
        RingEntry::new_receive(),
        RingEntry::new_receive(),
        RingEntry::new_receive(),
        RingEntry::new_receive(),
        RingEntry::new_receive(),
        RingEntry::new_receive(),
        RingEntry::new_receive(),
        RingEntry::new_receive(),
        RingEntry::new_receive(),
        RingEntry::new_receive(),
        RingEntry::new_receive(),
        RingEntry::new_receive(),
        RingEntry::new_receive(),
        RingEntry::new_receive(),
    ];
    static mut TRANSMIT_BUFFER: [RingEntry<Transmit>; 16] = [
        RingEntry::new_transmit(),
        RingEntry::new_transmit(),
        RingEntry::new_transmit(),
        RingEntry::new_transmit(),
        RingEntry::new_transmit(),
        RingEntry::new_transmit(),
        RingEntry::new_transmit(),
        RingEntry::new_transmit(),
        RingEntry::new_transmit(),
        RingEntry::new_transmit(),
        RingEntry::new_transmit(),
        RingEntry::new_transmit(),
        RingEntry::new_transmit(),
        RingEntry::new_transmit(),
        RingEntry::new_transmit(),
        RingEntry::new_transmit(),
    ];
    static mut ICMP_RECEIVE_METADATA_BUFFER: [IcmpPacketMetadata; 16] =
        [IcmpPacketMetadata::EMPTY; 16];
    static mut ICMP_TRANSMIT_METADATA_BUFFER: [IcmpPacketMetadata; 16] =
        [IcmpPacketMetadata::EMPTY; 16];
    static mut ICMP_RECEIVE_PAYLOAD_BUFFER: [u8; 1024] = [0; 1024];
    static mut ICMP_TRANSMIT_PAYLOAD_BUFFER: [u8; 1024] = [0; 1024];

    // Safety: We have checked that the buffers are only handed out once,
    // Therefore we can give out mutable static references to the buffers.
    //
    // We need this unsafe, to avoid the buffers being allocated on the stack
    unsafe {
        Some(Buffers {
            receive_buffer: &mut RECEIVE_BUFFER,
            transmit_buffer: &mut TRANSMIT_BUFFER,
            icmp_receive_metadata_buffer: &mut ICMP_RECEIVE_METADATA_BUFFER,
            icmp_transmit_metadata_buffer: &mut ICMP_TRANSMIT_METADATA_BUFFER,
            icmp_receive_payload_buffer: &mut ICMP_RECEIVE_PAYLOAD_BUFFER,
            icmp_transmit_payload_buffer: &mut ICMP_TRANSMIT_PAYLOAD_BUFFER,
        })
    }
}
