//! Env Switcher - A lightweight multi-version environment manager
//!
//! GUI Application for managing Node.js and Java environments

// Windows 子系统设置：不显示控制台窗口
#![windows_subsystem = "windows"]

mod gui;

fn main() {
    gui::run_gui().expect("Failed to run GUI application");
}
