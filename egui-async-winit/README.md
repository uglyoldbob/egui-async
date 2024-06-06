# egui-winit

[![Latest version](https://img.shields.io/crates/v/egui-async-winit.svg)](https://crates.io/crates/egui-async-winit)
[![Documentation](https://docs.rs/egui-async-winit/badge.svg)](https://docs.rs/egui-async-winit)
![MIT](https://img.shields.io/badge/license-MIT-blue.svg)
![Apache](https://img.shields.io/badge/license-Apache-blue.svg)

This crates provides async bindings between [`egui`](https://github.com/emilk/egui) and [`async-winit`](https://crates.io/crates/async-winit).

The library translates async-winit events to egui, handled copy/paste, updates the cursor, open links clicked in egui, etc.
