# thesis-rust

Eine Sammlung allen Rust-Codes meiner Thesis.
Folgende Projekte sind enthalten:

- bezier: Eine Reimplementierung des Bezier-Benchmarks des EEMBC in Rust, sowohl als Floating-Point als auch als Fixed-Point-Variante.
  Die Originalimplementierung in C ist hier zu finden: https://github.com/eembc/oabenchv2/tree/7fb9a2835ed59675f6f5b2c5553a55b8322dc5b9/oav2/bezierv2

- stm32f207-hal eine HAL-Crate für stm32f207 Mikroprozessoren, inklusive eines smoltcp-kompatiblen Ethernet-Treibers.
  Ein Ping-Client kann als Beispiel mit folgendem Befehl gebaut werden:
  
      cargo build -p stm32f207-hal --example ping --features=ethernet
    

- semihosting-files eine Bibliothek zum Ein-und Auslesen von Dateien über Semihosting, für beliebige Cortex-Prozessoren
