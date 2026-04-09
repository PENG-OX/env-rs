//! Env Switcher GUI - WSL UI Style
//!
//! 深色卡片式现代化界面

use anyhow::Result;
use env_switcher::{Config, HookInjector, PathMatcher};
use eframe::egui;
use std::path::PathBuf;

// ============ 颜色定义 ============

struct Colors;
impl Colors {
    // 背景色 - WSL UI 风格
    const BG_PRIMARY: egui::Color32 = egui::Color32::from_rgb(30, 30, 30);      // #1e1e1e 主背景
    const BG_SECONDARY: egui::Color32 = egui::Color32::from_rgb(25, 25, 25);    // #191919 次级背景
    const BG_CARD: egui::Color32 = egui::Color32::from_rgb(45, 45, 45);         // #2d2d2d 卡片背景
    const BG_CARD_HOVER: egui::Color32 = egui::Color32::from_rgb(55, 55, 55);   // #373737 悬停
    const BG_BUTTON: egui::Color32 = egui::Color32::from_rgb(35, 35, 35);       // #232323 按钮
    const BG_INPUT: egui::Color32 = egui::Color32::from_rgb(38, 38, 38);        // #262626 输入框

    // 边框色
    const BORDER_DEFAULT: egui::Color32 = egui::Color32::from_rgb(60, 60, 60);  // #3c3c3c
    const BORDER_SUBTLE: egui::Color32 = egui::Color32::from_rgb(50, 50, 50);   // #323232

    // 文字色
    const TEXT_PRIMARY: egui::Color32 = egui::Color32::from_rgb(255, 255, 255);
    const TEXT_SECONDARY: egui::Color32 = egui::Color32::from_rgb(180, 180, 180);
    const TEXT_MUTED: egui::Color32 = egui::Color32::from_rgb(120, 120, 120);

    // 状态色
    const SUCCESS: egui::Color32 = egui::Color32::from_rgb(0, 189, 99);         // #00bd63 成功绿
    const WARNING: egui::Color32 = egui::Color32::from_rgb(255, 184, 0);        // #ffb800 警告黄
    const ERROR: egui::Color32 = egui::Color32::from_rgb(255, 86, 108);         // #ff566c 错误红
    const INFO: egui::Color32 = egui::Color32::from_rgb(64, 169, 255);          // #40a9ff 信息蓝

    // 品牌色
    const NODE_GREEN: egui::Color32 = egui::Color32::from_rgb(90, 163, 67);     // #5aa343
    const JAVA_RED: egui::Color32 = egui::Color32::from_rgb(244, 88, 58);       // #f4583a

    // 按钮色
    const BUTTON_PRIMARY: egui::Color32 = egui::Color32::from_rgb(0, 189, 99);  // #00bd63 主按钮绿
    const BUTTON_DANGER: egui::Color32 = egui::Color32::from_rgb(255, 86, 108); // #ff566c 危险按钮
}

// ============ 应用状态 ============

#[derive(Default, Clone)]
pub struct VersionEntry {
    pub version: String,
    pub path: String,
}

#[derive(Default, Clone)]
pub struct PathMappingEntry {
    pub path: String,
    pub node_version: String,
    pub java_version: String,
}

#[derive(Default, Clone)]
pub struct StatusMessage {
    pub text: String,
    pub level: StatusLevel,
}

#[derive(Default, Clone, PartialEq)]
pub enum StatusLevel {
    #[default]
    Info,
    Success,
    Warning,
    Error,
}

// ============ 主应用 ============

pub struct EnvSwitcherApp {
    config: Config,
    #[allow(dead_code)]
    config_path: PathBuf,

    status_message: Option<StatusMessage>,
    hook_installed: bool,
    current_path: String,
    current_status: PathMatchStatus,

    // Node.js
    node_versions: Vec<VersionEntry>,
    new_node_version: String,
    new_node_path: String,
    selected_node_for_mapping: String,

    // Java
    java_versions: Vec<VersionEntry>,
    new_java_version: String,
    new_java_path: String,
    selected_java_for_mapping: String,

    // 路径映射
    path_mappings: Vec<PathMappingEntry>,
    new_mapping_path: String,

    // 对话框
    show_add_node_dialog: bool,
    show_add_java_dialog: bool,
    show_add_mapping_dialog: bool,

    // 激活的标签页
    active_tab: Tab,
}

#[derive(Default, Clone, Copy, PartialEq)]
enum Tab {
    #[default]
    Overview,
    Node,
    Java,
    Paths,
}

#[derive(Default, Clone)]
struct PathMatchStatus {
    matched_path: Option<String>,
    node_version: Option<String>,
    java_version: Option<String>,
    is_inherited: bool,
}

impl EnvSwitcherApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let config = Config::load().unwrap_or_default();
        let config_path = Config::default_config_path();

        let node_versions: Vec<VersionEntry> = config.node_versions
            .iter()
            .map(|(v, c)| VersionEntry { version: v.clone(), path: c.path.clone() })
            .collect();

        let java_versions: Vec<VersionEntry> = config.java_versions
            .iter()
            .map(|(v, c)| VersionEntry { version: v.clone(), path: c.path.clone() })
            .collect();

        let path_mappings: Vec<PathMappingEntry> = config.path_mappings
            .iter()
            .map(|m| PathMappingEntry {
                path: m.path.clone(),
                node_version: m.node_version.clone().unwrap_or_default(),
                java_version: m.java_version.clone().unwrap_or_default(),
            })
            .collect();

        let injector = HookInjector::new();
        let hook_installed = injector.is_installed();

        let current_path = std::env::current_dir()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_default();

        let matcher = PathMatcher::new(config.clone());
        let match_result = matcher.find_match(std::path::Path::new(&current_path));
        let current_status = PathMatchStatus {
            matched_path: match_result.matched_path,
            node_version: match_result.node_version,
            java_version: match_result.java_version,
            is_inherited: match_result.is_inherited,
        };

        Self {
            config,
            config_path,
            status_message: None,
            hook_installed,
            current_path,
            current_status,
            node_versions,
            new_node_version: String::new(),
            new_node_path: String::new(),
            selected_node_for_mapping: String::new(),
            java_versions,
            new_java_version: String::new(),
            new_java_path: String::new(),
            selected_java_for_mapping: String::new(),
            path_mappings,
            new_mapping_path: String::new(),
            show_add_node_dialog: false,
            show_add_java_dialog: false,
            show_add_mapping_dialog: false,
            active_tab: Tab::Overview,
        }
    }

    fn set_status(&mut self, text: impl Into<String>, level: StatusLevel) {
        self.status_message = Some(StatusMessage { text: text.into(), level });
    }

    fn save_config(&mut self) -> Result<()> {
        self.config.save()?;
        Ok(())
    }

    fn refresh_config(&mut self) {
        match Config::load() {
            Ok(config) => {
                self.config = config.clone();
                self.node_versions = config.node_versions.iter()
                    .map(|(v, c)| VersionEntry { version: v.clone(), path: c.path.clone() }).collect();
                self.java_versions = config.java_versions.iter()
                    .map(|(v, c)| VersionEntry { version: v.clone(), path: c.path.clone() }).collect();
                self.path_mappings = config.path_mappings.iter()
                    .map(|m| PathMappingEntry {
                        path: m.path.clone(),
                        node_version: m.node_version.clone().unwrap_or_default(),
                        java_version: m.java_version.clone().unwrap_or_default(),
                    }).collect();
                self.set_status("配置已刷新", StatusLevel::Success);
            }
            Err(e) => self.set_status(format!("加载配置失败：{}", e), StatusLevel::Error),
        }
    }

    fn install_hook(&mut self) {
        let injector = HookInjector::new();
        match injector.install() {
            Ok(result) => {
                self.hook_installed = true;
                if result.newly_installed {
                    self.set_status("Hook 已安装，请重启 PowerShell 生效", StatusLevel::Success);
                } else {
                    self.set_status("Hook 已安装", StatusLevel::Info);
                }
            }
            Err(e) => self.set_status(format!("安装 Hook 失败：{}", e), StatusLevel::Error),
        }
    }

    fn uninstall_hook(&mut self) {
        let injector = HookInjector::new();
        match injector.uninstall() {
            Ok(()) => {
                self.hook_installed = false;
                self.set_status("Hook 已卸载", StatusLevel::Success);
            }
            Err(e) => self.set_status(format!("卸载 Hook 失败：{}", e), StatusLevel::Error),
        }
    }

    fn add_node_version(&mut self) {
        if self.new_node_version.is_empty() || self.new_node_path.is_empty() {
            self.set_status("版本号和路径不能为空", StatusLevel::Warning);
            return;
        }
        self.config.add_node_version(self.new_node_version.clone(), self.new_node_path.clone());
        if let Err(e) = self.save_config() {
            self.set_status(format!("保存配置失败：{}", e), StatusLevel::Error);
        } else {
            self.node_versions.push(VersionEntry {
                version: self.new_node_version.clone(),
                path: self.new_node_path.clone(),
            });
            self.new_node_version.clear();
            self.new_node_path.clear();
            self.show_add_node_dialog = false;
            self.set_status(format!("已添加 Node.js {}", self.new_node_version.clone()), StatusLevel::Success);
        }
    }

    fn remove_node_version(&mut self, version: &str) {
        self.config.node_versions.remove(version);
        self.node_versions.retain(|v| v.version != version);
        if let Err(e) = self.save_config() {
            self.set_status(format!("保存配置失败：{}", e), StatusLevel::Error);
        } else {
            self.set_status(format!("已移除 Node.js {}", version), StatusLevel::Success);
        }
    }

    fn add_java_version(&mut self) {
        if self.new_java_version.is_empty() || self.new_java_path.is_empty() {
            self.set_status("版本号和路径不能为空", StatusLevel::Warning);
            return;
        }
        self.config.add_java_version(self.new_java_version.clone(), self.new_java_path.clone());
        if let Err(e) = self.save_config() {
            self.set_status(format!("保存配置失败：{}", e), StatusLevel::Error);
        } else {
            self.java_versions.push(VersionEntry {
                version: self.new_java_version.clone(),
                path: self.new_java_path.clone(),
            });
            self.new_java_version.clear();
            self.new_java_path.clear();
            self.show_add_java_dialog = false;
            self.set_status(format!("已添加 Java {}", self.new_java_version.clone()), StatusLevel::Success);
        }
    }

    fn remove_java_version(&mut self, version: &str) {
        self.config.java_versions.remove(version);
        self.java_versions.retain(|v| v.version != version);
        if let Err(e) = self.save_config() {
            self.set_status(format!("保存配置失败：{}", e), StatusLevel::Error);
        } else {
            self.set_status(format!("已移除 Java {}", version), StatusLevel::Success);
        }
    }

    fn add_path_mapping(&mut self) {
        if self.new_mapping_path.is_empty() {
            self.set_status("项目路径不能为空", StatusLevel::Warning);
            return;
        }
        let node_ver = if self.selected_node_for_mapping.is_empty() { None } else { Some(self.selected_node_for_mapping.clone()) };
        let java_ver = if self.selected_java_for_mapping.is_empty() { None } else { Some(self.selected_java_for_mapping.clone()) };

        self.config.add_path_mapping(self.new_mapping_path.clone(), node_ver, java_ver);
        if let Err(e) = self.save_config() {
            self.set_status(format!("保存配置失败：{}", e), StatusLevel::Error);
        } else {
            self.path_mappings.push(PathMappingEntry {
                path: self.new_mapping_path.clone(),
                node_version: self.selected_node_for_mapping.clone(),
                java_version: self.selected_java_for_mapping.clone(),
            });
            self.new_mapping_path.clear();
            self.selected_node_for_mapping.clear();
            self.selected_java_for_mapping.clear();
            self.show_add_mapping_dialog = false;
            self.set_status(format!("已添加路径映射：{}", self.new_mapping_path.clone()), StatusLevel::Success);
        }
    }

    fn remove_path_mapping(&mut self, path: &str) {
        self.config.remove_path_mapping(path);
        self.path_mappings.retain(|m| m.path != path);
        if let Err(e) = self.save_config() {
            self.set_status(format!("保存配置失败：{}", e), StatusLevel::Error);
        } else {
            self.set_status(format!("已移除路径映射：{}", path), StatusLevel::Success);
        }
    }

    fn open_config_folder(&mut self) {
        let path = Config::default_config_path();
        if path.exists() {
            std::process::Command::new("explorer").arg("/select,").arg(&path).spawn().ok();
            self.set_status("已打开配置文件夹", StatusLevel::Success);
        } else {
            self.set_status("配置文件不存在", StatusLevel::Warning);
        }
    }

    fn browse_for_folder(&self, title: &str) -> Option<String> {
        rfd::FileDialog::new().set_title(title).pick_folder().map(|p| p.to_string_lossy().to_string())
    }

    // ============ UI 组件 ============

    fn render_card<F>(&mut self, ui: &mut egui::Ui, title: impl Into<String>, add_content: F)
    where F: FnOnce(&mut Self, &mut egui::Ui)
    {
        let frame = egui::Frame::none()
            .fill(Colors::BG_CARD)
            .stroke(egui::Stroke::new(1.0, Colors::BORDER_DEFAULT))
            .corner_radius(12);

        frame.show(ui, |ui| {
            ui.vertical(|ui| {
                // 标题栏
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new(title.into()).size(16.0).color(Colors::TEXT_PRIMARY));
                });
                ui.add_space(8.0);
                add_content(self, ui);
            });
        });
    }

    fn render_status_indicator(&self, ui: &mut egui::Ui, status: &str, is_active: bool) {
        let (color, text) = if is_active {
            (Colors::SUCCESS, "● 运行中")
        } else {
            (Colors::TEXT_MUTED, "○ 未激活")
        };

        ui.horizontal(|ui| {
            ui.colored_label(color, text);
            ui.label(egui::RichText::new(status).color(Colors::TEXT_SECONDARY).size(14.0));
        });
    }
}

impl eframe::App for EnvSwitcherApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.request_repaint_after(std::time::Duration::from_secs(1));

        // 设置全局样式 - WSL UI 风格
        let mut style = (*ctx.style()).clone();
        style.visuals.window_fill = Colors::BG_PRIMARY;
        style.visuals.panel_fill = Colors::BG_PRIMARY;
        style.visuals.widgets.inactive.bg_fill = Colors::BG_BUTTON;
        style.visuals.widgets.hovered.bg_fill = Colors::BG_CARD_HOVER;
        style.visuals.widgets.active.bg_fill = Colors::BG_CARD;
        style.visuals.selection.bg_fill = Colors::SUCCESS;
        style.visuals.window_stroke = egui::Stroke::new(1.0, Colors::BORDER_DEFAULT);
        ctx.set_style(style);

        // 顶部工具栏
        egui::TopBottomPanel::top("header").show(ctx, |ui| {
            ui.add_space(10.0);
            ui.horizontal(|ui| {
                // Logo
                ui.label(egui::RichText::new("🔌").size(20.0));
                ui.add_space(4.0);
                ui.vertical(|ui| {
                    ui.label(egui::RichText::new("Env Switcher").size(16.0).color(Colors::TEXT_PRIMARY));
                    ui.label(egui::RichText::new("多版本环境管理工具").size(10.0).color(Colors::TEXT_MUTED));
                });

                ui.add_space(20.0);

                // 功能按钮 - WSL UI 风格
                let button_style = |ui: &mut egui::Ui, text: &str, color: egui::Color32| {
                    let btn = egui::Button::new(
                        egui::RichText::new(text).size(12.0).color(Colors::TEXT_PRIMARY)
                    )
                    .fill(Colors::BG_BUTTON)
                    .stroke(egui::Stroke::new(1.0, color))
                    .rounding(6);
                    ui.add(btn)
                };

                if button_style(ui, "+ 添加", Colors::SUCCESS).clicked() {
                    self.show_add_node_dialog = true;
                }
                ui.add_space(8.0);
                if button_style(ui, " 打开配置", Colors::WARNING).clicked() {
                    self.open_config_folder();
                }
                ui.add_space(8.0);
                if button_style(ui, "⟳ 刷新", Colors::INFO).clicked() {
                    self.refresh_config();
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button(egui::RichText::new("⚙").size(16.0)).clicked() {
                        // 设置按钮
                    }
                });
            });
            ui.add_space(10.0);
        });

        egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                if let Some(ref status) = self.status_message {
                    let color = match status.level {
                        StatusLevel::Info => Colors::INFO,
                        StatusLevel::Success => Colors::SUCCESS,
                        StatusLevel::Warning => Colors::WARNING,
                        StatusLevel::Error => Colors::ERROR,
                    };
                    ui.colored_label(color, &status.text);
                } else {
                    ui.colored_label(Colors::SUCCESS, "✓ 就绪");
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(egui::RichText::new(format!("v{}", env!("CARGO_PKG_VERSION"))).color(Colors::TEXT_MUTED).size(11.0));
                });
            });
            ui.add_space(4.0);
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.add_space(16.0);

            egui::ScrollArea::vertical().show(ui, |ui| {
                if self.active_tab == Tab::Overview {
                    self.render_overview(ui);
                } else if self.active_tab == Tab::Node {
                    self.render_node_tab(ui);
                } else if self.active_tab == Tab::Java {
                    self.render_java_tab(ui);
                } else if self.active_tab == Tab::Paths {
                    self.render_paths_tab(ui);
                }
            });
        });

        // 对话框
        self.render_dialogs(ctx);
    }
}

impl EnvSwitcherApp {
    fn render_overview(&mut self, ui: &mut egui::Ui) {
        // 状态卡片行
        ui.horizontal(|ui| {
            // Hook 状态卡片
            let card_size = egui::vec2(220.0, 140.0);
            ui.vertical(|ui| {
                let frame = egui::Frame::none()
                    .fill(Colors::BG_CARD)
                    .stroke(egui::Stroke::new(1.0, Colors::BORDER_DEFAULT))
                    .corner_radius(8);

                frame.show(ui, |ui| {
                    ui.set_min_size(card_size);
                    ui.vertical(|ui| {
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new("🔌").size(20.0));
                            ui.label(egui::RichText::new("Hook 状态").size(14.0).color(Colors::TEXT_PRIMARY));
                        });
                        ui.add_space(12.0);

                        if self.hook_installed {
                            ui.colored_label(Colors::SUCCESS, "● 已安装");
                            ui.label(egui::RichText::new("运行中").color(Colors::TEXT_SECONDARY).size(12.0));
                        } else {
                            ui.colored_label(Colors::TEXT_MUTED, "○ 未安装");
                            ui.label(egui::RichText::new("未运行").color(Colors::TEXT_SECONDARY).size(12.0));
                        }

                        ui.add_space(16.0);
                        ui.horizontal(|ui| {
                            let reinstall_btn = egui::Button::new(
                                egui::RichText::new("重装").size(11.0).color(Colors::TEXT_PRIMARY)
                            )
                            .fill(Colors::BG_BUTTON)
                            .stroke(egui::Stroke::new(1.0, Colors::SUCCESS))
                            .rounding(6);
                            if ui.add(reinstall_btn).clicked() { self.install_hook(); }

                            if self.hook_installed {
                                let uninstall_btn = egui::Button::new(
                                    egui::RichText::new("卸载").size(11.0).color(Colors::TEXT_PRIMARY)
                                )
                                .fill(Colors::BG_BUTTON)
                                .stroke(egui::Stroke::new(1.0, Colors::ERROR))
                                .rounding(6);
                                if ui.add(uninstall_btn).clicked() { self.uninstall_hook(); }
                            }
                        });
                    });
                });
            });

            // Node.js 卡片
            ui.vertical(|ui| {
                let frame = egui::Frame::none()
                    .fill(Colors::BG_CARD)
                    .stroke(egui::Stroke::new(1.0, Colors::BORDER_DEFAULT))
                    .corner_radius(8);

                frame.show(ui, |ui| {
                    ui.set_min_size(card_size);
                    ui.vertical(|ui| {
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new("⬢").size(20.0).color(Colors::NODE_GREEN));
                            ui.label(egui::RichText::new("Node.js").size(14.0).color(Colors::TEXT_PRIMARY));
                        });
                        ui.add_space(12.0);

                        ui.label(egui::RichText::new(format!("{} 个版本", self.node_versions.len()))
                            .color(Colors::TEXT_PRIMARY).size(28.0));
                        ui.label(egui::RichText::new(format!("{} 个项目", self.path_mappings.iter()
                            .filter(|m| !m.node_version.is_empty()).count()))
                            .color(Colors::TEXT_SECONDARY).size(12.0));

                        ui.add_space(16.0);
                        let manage_btn = egui::Button::new(
                            egui::RichText::new("管理 →").size(11.0).color(Colors::TEXT_PRIMARY)
                        )
                        .fill(Colors::BG_BUTTON)
                        .stroke(egui::Stroke::new(1.0, Colors::NODE_GREEN))
                        .rounding(6);
                        if ui.add(manage_btn).clicked() { self.active_tab = Tab::Node; }
                    });
                });
            });

            // Java 卡片
            ui.vertical(|ui| {
                let frame = egui::Frame::none()
                    .fill(Colors::BG_CARD)
                    .stroke(egui::Stroke::new(1.0, Colors::BORDER_DEFAULT))
                    .corner_radius(8);

                frame.show(ui, |ui| {
                    ui.set_min_size(card_size);
                    ui.vertical(|ui| {
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new("☕").size(20.0).color(Colors::JAVA_RED));
                            ui.label(egui::RichText::new("Java").size(14.0).color(Colors::TEXT_PRIMARY));
                        });
                        ui.add_space(12.0);

                        ui.label(egui::RichText::new(format!("{} 个版本", self.java_versions.len()))
                            .color(Colors::TEXT_PRIMARY).size(28.0));
                        ui.label(egui::RichText::new(format!("{} 个项目", self.path_mappings.iter()
                            .filter(|m| !m.java_version.is_empty()).count()))
                            .color(Colors::TEXT_SECONDARY).size(12.0));

                        ui.add_space(16.0);
                        let manage_btn = egui::Button::new(
                            egui::RichText::new("管理 →").size(11.0).color(Colors::TEXT_PRIMARY)
                        )
                        .fill(Colors::BG_BUTTON)
                        .stroke(egui::Stroke::new(1.0, Colors::JAVA_RED))
                        .rounding(6);
                        if ui.add(manage_btn).clicked() { self.active_tab = Tab::Java; }
                    });
                });
            });
        });

        ui.add_space(20.0);

        // 当前环境卡片
        let frame = egui::Frame::none()
            .fill(Colors::BG_CARD)
            .stroke(egui::Stroke::new(1.0, Colors::BORDER_DEFAULT))
            .corner_radius(8);

        frame.show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("📂").size(18.0));
                ui.label(egui::RichText::new("当前路径").size(14.0).color(Colors::TEXT_PRIMARY));
                ui.add_space(8.0);
                ui.label(egui::RichText::new(&self.current_path)
                    .color(Colors::TEXT_SECONDARY).size(12.0));

                if self.current_status.matched_path.is_some() {
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(egui::RichText::new("● 已激活").color(Colors::SUCCESS).size(12.0));
                    });
                }
            });

            ui.add_space(10.0);
            ui.separator();
            ui.add_space(10.0);

            ui.horizontal(|ui| {
                if let Some(ref node) = self.current_status.node_version {
                    ui.colored_label(Colors::NODE_GREEN, format!("⬢ Node: v{}", node));
                } else {
                    ui.colored_label(Colors::TEXT_MUTED, "⬢ Node: 未配置");
                }

                ui.add_space(30.0);

                if let Some(ref java) = self.current_status.java_version {
                    ui.colored_label(Colors::JAVA_RED, format!("☕ Java: v{}", java));
                } else {
                    ui.colored_label(Colors::TEXT_MUTED, "☕ Java: 未配置");
                }
            });
        });
    }

    fn render_node_tab(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("⬢").size(20.0).color(Colors::NODE_GREEN));
            ui.label(egui::RichText::new("Node.js 版本").size(16.0).color(Colors::TEXT_PRIMARY));

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let add_btn = egui::Button::new(
                    egui::RichText::new("+ 添加版本").size(11.0).color(Colors::TEXT_PRIMARY)
                )
                .fill(Colors::BG_BUTTON)
                .stroke(egui::Stroke::new(1.0, Colors::SUCCESS))
                .rounding(6);
                if ui.add(add_btn).clicked() { self.show_add_node_dialog = true; }
            });
        });

        ui.add_space(12.0);

        for entry in self.node_versions.clone() {
            let frame = egui::Frame::none()
                .fill(Colors::BG_CARD)
                .stroke(egui::Stroke::new(1.0, Colors::BORDER_DEFAULT))
                .corner_radius(8);

            frame.show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        ui.label(egui::RichText::new(format!("v{}", entry.version))
                            .color(Colors::NODE_GREEN).size(15.0));
                        ui.label(egui::RichText::new(&entry.path)
                            .color(Colors::TEXT_SECONDARY).size(11.0));
                    });

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let del_btn = egui::Button::new(
                            egui::RichText::new("🗑").size(14.0)
                        )
                        .fill(Colors::BG_BUTTON)
                        .stroke(egui::Stroke::new(1.0, Colors::ERROR))
                        .rounding(6);
                        if ui.add(del_btn).clicked() { self.remove_node_version(&entry.version); }
                    });
                });
            });
            ui.add_space(6.0);
        }
    }

    fn render_java_tab(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("☕").size(20.0).color(Colors::JAVA_RED));
            ui.label(egui::RichText::new("Java 版本").size(16.0).color(Colors::TEXT_PRIMARY));

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let add_btn = egui::Button::new(
                    egui::RichText::new("+ 添加版本").size(11.0).color(Colors::TEXT_PRIMARY)
                )
                .fill(Colors::BG_BUTTON)
                .stroke(egui::Stroke::new(1.0, Colors::SUCCESS))
                .rounding(6);
                if ui.add(add_btn).clicked() { self.show_add_java_dialog = true; }
            });
        });

        ui.add_space(12.0);

        for entry in self.java_versions.clone() {
            let frame = egui::Frame::none()
                .fill(Colors::BG_CARD)
                .stroke(egui::Stroke::new(1.0, Colors::BORDER_DEFAULT))
                .corner_radius(8);

            frame.show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        ui.label(egui::RichText::new(format!("v{}", entry.version))
                            .color(Colors::JAVA_RED).size(15.0));
                        ui.label(egui::RichText::new(&entry.path)
                            .color(Colors::TEXT_SECONDARY).size(11.0));
                    });

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let del_btn = egui::Button::new(
                            egui::RichText::new("🗑").size(14.0)
                        )
                        .fill(Colors::BG_BUTTON)
                        .stroke(egui::Stroke::new(1.0, Colors::ERROR))
                        .rounding(6);
                        if ui.add(del_btn).clicked() { self.remove_java_version(&entry.version); }
                    });
                });
            });
            ui.add_space(6.0);
        }
    }

    fn render_paths_tab(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("📁").size(20.0));
            ui.label(egui::RichText::new("路径映射").size(16.0).color(Colors::TEXT_PRIMARY));

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let add_btn = egui::Button::new(
                    egui::RichText::new("+ 添加映射").size(11.0).color(Colors::TEXT_PRIMARY)
                )
                .fill(Colors::BG_BUTTON)
                .stroke(egui::Stroke::new(1.0, Colors::SUCCESS))
                .rounding(6);
                if ui.add(add_btn).clicked() { self.show_add_mapping_dialog = true; }
            });
        });

        ui.add_space(12.0);

        for entry in self.path_mappings.clone() {
            let frame = egui::Frame::none()
                .fill(Colors::BG_CARD)
                .stroke(egui::Stroke::new(1.0, Colors::BORDER_DEFAULT))
                .corner_radius(8);

            frame.show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        ui.label(egui::RichText::new(format!("📁 {}", entry.path))
                            .color(Colors::TEXT_PRIMARY).size(13.0));
                        ui.label(egui::RichText::new(&entry.path)
                            .color(Colors::TEXT_SECONDARY).size(11.0));
                        ui.add_space(6.0);
                        ui.horizontal(|ui| {
                            if !entry.node_version.is_empty() {
                                ui.colored_label(Colors::NODE_GREEN, format!("⬢ {}", entry.node_version));
                            } else {
                                ui.colored_label(Colors::TEXT_MUTED, "⬢ 未配置");
                            }

                            ui.add_space(15.0);

                            if !entry.java_version.is_empty() {
                                ui.colored_label(Colors::JAVA_RED, format!("☕ {}", entry.java_version));
                            } else {
                                ui.colored_label(Colors::TEXT_MUTED, "☕ 未配置");
                            }
                        });
                    });

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let del_btn = egui::Button::new(
                            egui::RichText::new("🗑").size(14.0)
                        )
                        .fill(Colors::BG_BUTTON)
                        .stroke(egui::Stroke::new(1.0, Colors::ERROR))
                        .rounding(6);
                        if ui.add(del_btn).clicked() { self.remove_path_mapping(&entry.path); }
                    });
                });
            });
            ui.add_space(6.0);
        }
    }

    fn render_dialogs(&mut self, ctx: &egui::Context) {
        // Node.js 添加对话框
        if self.show_add_node_dialog {
            egui::Window::new("  添加 Node.js 版本")
                .collapsible(false)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .resizable(false)
                .show(ctx, |ui| {
                    ui.add_space(10.0);
                    ui.label(egui::RichText::new("版本号:").size(12.0).color(Colors::TEXT_SECONDARY));
                    ui.add_space(4.0);
                    ui.text_edit_singleline(&mut self.new_node_version);
                    ui.add_space(8.0);
                    ui.label(egui::RichText::new("安装路径:").size(12.0).color(Colors::TEXT_SECONDARY));
                    ui.add_space(4.0);
                    ui.horizontal(|ui| {
                        ui.text_edit_singleline(&mut self.new_node_path);
                        let browse_btn = egui::Button::new(
                            egui::RichText::new("浏览...").size(11.0).color(Colors::TEXT_PRIMARY)
                        )
                        .fill(Colors::BG_BUTTON)
                        .stroke(egui::Stroke::new(1.0, Colors::INFO))
                        .rounding(6);
                        if ui.add(browse_btn).clicked() {
                            if let Some(path) = self.browse_for_folder("选择 Node.js 安装目录") {
                                self.new_node_path = path;
                            }
                        }
                    });
                    ui.add_space(12.0);
                    ui.horizontal(|ui| {
                        let ok_btn = egui::Button::new(
                            egui::RichText::new("确定").size(11.0).color(Colors::TEXT_PRIMARY)
                        )
                        .fill(Colors::BG_BUTTON)
                        .stroke(egui::Stroke::new(1.0, Colors::SUCCESS))
                        .rounding(6);
                        let cancel_btn = egui::Button::new(
                            egui::RichText::new("取消").size(11.0).color(Colors::TEXT_PRIMARY)
                        )
                        .fill(Colors::BG_BUTTON)
                        .stroke(egui::Stroke::new(1.0, Colors::BORDER_DEFAULT))
                        .rounding(6);
                        if ui.add(ok_btn).clicked() { self.add_node_version(); }
                        if ui.add(cancel_btn).clicked() {
                            self.show_add_node_dialog = false;
                            self.new_node_version.clear();
                            self.new_node_path.clear();
                        }
                    });
                    ui.add_space(10.0);
                });
        }

        // Java 添加对话框
        if self.show_add_java_dialog {
            egui::Window::new("  添加 Java 版本")
                .collapsible(false)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .resizable(false)
                .show(ctx, |ui| {
                    ui.add_space(10.0);
                    ui.label(egui::RichText::new("版本号:").size(12.0).color(Colors::TEXT_SECONDARY));
                    ui.add_space(4.0);
                    ui.text_edit_singleline(&mut self.new_java_version);
                    ui.add_space(8.0);
                    ui.label(egui::RichText::new("安装路径:").size(12.0).color(Colors::TEXT_SECONDARY));
                    ui.add_space(4.0);
                    ui.horizontal(|ui| {
                        ui.text_edit_singleline(&mut self.new_java_path);
                        let browse_btn = egui::Button::new(
                            egui::RichText::new("浏览...").size(11.0).color(Colors::TEXT_PRIMARY)
                        )
                        .fill(Colors::BG_BUTTON)
                        .stroke(egui::Stroke::new(1.0, Colors::INFO))
                        .rounding(6);
                        if ui.add(browse_btn).clicked() {
                            if let Some(path) = self.browse_for_folder("选择 Java 安装目录") {
                                self.new_java_path = path;
                            }
                        }
                    });
                    ui.add_space(12.0);
                    ui.horizontal(|ui| {
                        let ok_btn = egui::Button::new(
                            egui::RichText::new("确定").size(11.0).color(Colors::TEXT_PRIMARY)
                        )
                        .fill(Colors::BG_BUTTON)
                        .stroke(egui::Stroke::new(1.0, Colors::SUCCESS))
                        .rounding(6);
                        let cancel_btn = egui::Button::new(
                            egui::RichText::new("取消").size(11.0).color(Colors::TEXT_PRIMARY)
                        )
                        .fill(Colors::BG_BUTTON)
                        .stroke(egui::Stroke::new(1.0, Colors::BORDER_DEFAULT))
                        .rounding(6);
                        if ui.add(ok_btn).clicked() { self.add_java_version(); }
                        if ui.add(cancel_btn).clicked() {
                            self.show_add_java_dialog = false;
                            self.new_java_version.clear();
                            self.new_java_path.clear();
                        }
                    });
                    ui.add_space(10.0);
                });
        }

        // 路径映射添加对话框
        if self.show_add_mapping_dialog {
            egui::Window::new("  添加路径映射")
                .collapsible(false)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .resizable(false)
                .show(ctx, |ui| {
                    ui.add_space(10.0);
                    ui.label(egui::RichText::new("项目路径:").size(12.0).color(Colors::TEXT_SECONDARY));
                    ui.add_space(4.0);
                    ui.horizontal(|ui| {
                        ui.text_edit_singleline(&mut self.new_mapping_path);
                        let browse_btn = egui::Button::new(
                            egui::RichText::new("浏览...").size(11.0).color(Colors::TEXT_PRIMARY)
                        )
                        .fill(Colors::BG_BUTTON)
                        .stroke(egui::Stroke::new(1.0, Colors::INFO))
                        .rounding(6);
                        if ui.add(browse_btn).clicked() {
                            if let Some(path) = self.browse_for_folder("选择项目目录") {
                                self.new_mapping_path = path;
                            }
                        }
                    });
                    ui.add_space(10.0);

                    ui.label(egui::RichText::new("Node.js 版本:").size(12.0).color(Colors::TEXT_SECONDARY));
                    ui.add_space(4.0);
                    egui::ComboBox::from_id_salt("node_select")
                        .selected_text(if self.selected_node_for_mapping.is_empty() { "无" } else { &self.selected_node_for_mapping })
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut self.selected_node_for_mapping, String::new(), "无");
                            for entry in &self.node_versions {
                                ui.selectable_value(&mut self.selected_node_for_mapping, entry.version.clone(), &entry.version);
                            }
                        });
                    ui.add_space(8.0);

                    ui.label(egui::RichText::new("Java 版本:").size(12.0).color(Colors::TEXT_SECONDARY));
                    ui.add_space(4.0);
                    egui::ComboBox::from_id_salt("java_select")
                        .selected_text(if self.selected_java_for_mapping.is_empty() { "无" } else { &self.selected_java_for_mapping })
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut self.selected_java_for_mapping, String::new(), "无");
                            for entry in &self.java_versions {
                                ui.selectable_value(&mut self.selected_java_for_mapping, entry.version.clone(), &entry.version);
                            }
                        });
                    ui.add_space(12.0);

                    ui.horizontal(|ui| {
                        let ok_btn = egui::Button::new(
                            egui::RichText::new("确定").size(11.0).color(Colors::TEXT_PRIMARY)
                        )
                        .fill(Colors::BG_BUTTON)
                        .stroke(egui::Stroke::new(1.0, Colors::SUCCESS))
                        .rounding(6);
                        let cancel_btn = egui::Button::new(
                            egui::RichText::new("取消").size(11.0).color(Colors::TEXT_PRIMARY)
                        )
                        .fill(Colors::BG_BUTTON)
                        .stroke(egui::Stroke::new(1.0, Colors::BORDER_DEFAULT))
                        .rounding(6);
                        if ui.add(ok_btn).clicked() { self.add_path_mapping(); }
                        if ui.add(cancel_btn).clicked() {
                            self.show_add_mapping_dialog = false;
                            self.new_mapping_path.clear();
                            self.selected_node_for_mapping.clear();
                            self.selected_java_for_mapping.clear();
                        }
                    });
                    ui.add_space(10.0);
                });
        }
    }
}

/// 运行 GUI 应用
pub fn run_gui() -> Result<()> {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1100.0, 750.0])
            .with_min_inner_size([900.0, 600.0])
            .with_title("Env Switcher"),
        ..Default::default()
    };

    eframe::run_native(
        "Env Switcher",
        native_options,
        Box::new(|cc| {
            // 设置中文字体
            let mut fonts = egui::FontDefinitions::default();
            #[cfg(target_os = "windows")]
            {
                use std::fs::File;
                use std::io::Read;
                let font_paths = ["C:\\Windows\\Fonts\\msyh.ttc", "C:\\Windows\\Fonts\\simsun.ttc"];
                for path in &font_paths {
                    if std::path::Path::new(path).exists() {
                        let mut buffer = Vec::new();
                        if File::open(path).and_then(|mut f| f.read_to_end(&mut buffer)).is_ok() {
                            fonts.font_data.insert("chinese".to_owned(), std::sync::Arc::new(egui::FontData::from_owned(buffer)));
                            fonts.families.entry(egui::FontFamily::Proportional).or_default().insert(0, "chinese".to_owned());
                            fonts.families.entry(egui::FontFamily::Monospace).or_default().push("chinese".to_owned());
                            break;
                        }
                    }
                }
            }
            cc.egui_ctx.set_fonts(fonts);

            Ok(Box::new(EnvSwitcherApp::new(cc)))
        }),
    ).map_err(|e| anyhow::anyhow!("{}", e))?;

    Ok(())
}
