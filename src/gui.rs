//! Env Switcher GUI - Windows 11 Fluent Style
//!
//! 使用 iced 框架实现的现代化界面

use anyhow::Result;
use env_switcher::{Config, HookInjector, PathMatcher};
use iced::widget::{
    button, column, container, horizontal_space, row, scrollable, text, text_input, Column, Row,
    Space, Text,
};
use iced::{window, Alignment, Border, Color, Element, Fill, Length, Task, Theme};
use iced_aw::{bootstrap::Bootstrap, BOOTSTRAP_FONT};
use std::path::PathBuf;

// ============ Fluent 风格颜色 ============

struct FluentColors;

impl FluentColors {
    const BG_BASE: Color = Color::from_rgb(0.125, 0.125, 0.125);
    const BG_CARD: Color = Color::from_rgb(0.165, 0.165, 0.165);
    const BG_BUTTON_PRIMARY: Color = Color::from_rgb(0.0, 0.749, 0.392);
    const BORDER_DEFAULT: Color = Color::from_rgb(0.235, 0.235, 0.235);
    const BORDER_SUBTLE: Color = Color::from_rgb(0.196, 0.196, 0.196);
    const TEXT_PRIMARY: Color = Color::from_rgb(1.0, 1.0, 1.0);
    const TEXT_SECONDARY: Color = Color::from_rgb(0.706, 0.706, 0.706);
    const TEXT_MUTED: Color = Color::from_rgb(0.471, 0.471, 0.471);
    const SUCCESS: Color = Color::from_rgb(0.0, 0.749, 0.392);
    const WARNING: Color = Color::from_rgb(1.0, 0.722, 0.0);
    const ERROR: Color = Color::from_rgb(1.0, 0.337, 0.424);
    const INFO: Color = Color::from_rgb(0.251, 0.663, 1.0);
    const NODE_GREEN: Color = Color::from_rgb(0.353, 0.639, 0.263);
    const JAVA_RED: Color = Color::from_rgb(0.957, 0.349, 0.227);
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

#[derive(Default, Clone, PartialEq)]
pub enum StatusLevel {
    #[default]
    Info,
    Success,
    Warning,
    Error,
}

#[derive(Clone)]
pub struct StatusMessage {
    pub text: String,
    pub level: StatusLevel,
}

// ============ 主应用 ============

#[derive(Default)]
pub struct EnvSwitcherApp {
    config: Config,
    config_path: PathBuf,
    status_message: Option<StatusMessage>,
    hook_installed: bool,
    current_path: String,
    current_status: PathMatchStatus,
    node_versions: Vec<VersionEntry>,
    new_node_version: String,
    new_node_path: String,
    selected_node_for_mapping: String,
    java_versions: Vec<VersionEntry>,
    new_java_version: String,
    new_java_path: String,
    selected_java_for_mapping: String,
    path_mappings: Vec<PathMappingEntry>,
    new_mapping_path: String,
    active_tab: Tab,
    show_add_node_dialog: bool,
    show_add_java_dialog: bool,
    show_add_mapping_dialog: bool,
}

#[derive(Default, Clone, Copy, PartialEq, Debug)]
enum Tab {
    #[default]
    Overview,
    Node,
    Java,
    Paths,
}

impl Tab {
    fn all() -> &'static [Tab] {
        &[Tab::Overview, Tab::Node, Tab::Java, Tab::Paths]
    }

    fn label(&self) -> &'static str {
        match self {
            Tab::Overview => "概览",
            Tab::Node => "Node.js",
            Tab::Java => "Java",
            Tab::Paths => "路径映射",
        }
    }
}

#[derive(Default, Clone)]
struct PathMatchStatus {
    matched_path: Option<String>,
    node_version: Option<String>,
    java_version: Option<String>,
    is_inherited: bool,
}

// ============ 消息定义 ============

#[derive(Debug, Clone)]
pub enum Message {
    TabSelected(Tab),
    InstallHook,
    UninstallHook,
    AddNodeVersion,
    RemoveNodeVersion(String),
    OpenAddNodeDialog,
    CloseAddNodeDialog,
    NodeVersionInputChanged(String),
    NodePathInputChanged(String),
    BrowseNodePath,
    NodeVersionInputForMappingChanged(String),
    AddJavaVersion,
    RemoveJavaVersion(String),
    OpenAddJavaDialog,
    CloseAddJavaDialog,
    JavaVersionInputChanged(String),
    JavaPathInputChanged(String),
    BrowseJavaPath,
    JavaVersionInputForMappingChanged(String),
    AddPathMapping,
    RemovePathMapping(String),
    OpenAddMappingDialog,
    CloseAddMappingDialog,
    MappingPathInputChanged(String),
    BrowseMappingPath,
    RefreshConfig,
    OpenConfigFolder,
}

impl EnvSwitcherApp {
    pub fn new() -> (Self, Task<Message>) {
        let config = Config::load().unwrap_or_default();
        let config_path = Config::default_config_path();

        let node_versions: Vec<VersionEntry> = config
            .node_versions
            .iter()
            .map(|(v, c)| VersionEntry {
                version: v.clone(),
                path: c.path.clone(),
            })
            .collect();

        let java_versions: Vec<VersionEntry> = config
            .java_versions
            .iter()
            .map(|(v, c)| VersionEntry {
                version: v.clone(),
                path: c.path.clone(),
            })
            .collect();

        let path_mappings: Vec<PathMappingEntry> = config
            .path_mappings
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

        let app = Self {
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
            active_tab: Tab::Overview,
            show_add_node_dialog: false,
            show_add_java_dialog: false,
            show_add_mapping_dialog: false,
        };

        (app, Task::none())
    }

    fn set_status(&mut self, text: impl Into<String>, level: StatusLevel) {
        self.status_message = Some(StatusMessage {
            text: text.into(),
            level,
        });
    }

    fn save_config(&mut self) -> Result<()> {
        self.config.save()?;
        Ok(())
    }

    fn refresh_config(&mut self) {
        match Config::load() {
            Ok(config) => {
                self.config = config.clone();
                self.node_versions = config
                    .node_versions
                    .iter()
                    .map(|(v, c)| VersionEntry {
                        version: v.clone(),
                        path: c.path.clone(),
                    })
                    .collect();
                self.java_versions = config
                    .java_versions
                    .iter()
                    .map(|(v, c)| VersionEntry {
                        version: v.clone(),
                        path: c.path.clone(),
                    })
                    .collect();
                self.path_mappings = config
                    .path_mappings
                    .iter()
                    .map(|m| PathMappingEntry {
                        path: m.path.clone(),
                        node_version: m.node_version.clone().unwrap_or_default(),
                        java_version: m.java_version.clone().unwrap_or_default(),
                    })
                    .collect();
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
        let version = self.new_node_version.clone();
        self.config
            .add_node_version(version.clone(), self.new_node_path.clone());
        if let Err(e) = self.save_config() {
            self.set_status(format!("保存配置失败：{}", e), StatusLevel::Error);
        } else {
            self.node_versions.push(VersionEntry {
                version: version.clone(),
                path: self.new_node_path.clone(),
            });
            self.new_node_version.clear();
            self.new_node_path.clear();
            self.show_add_node_dialog = false;
            self.set_status(format!("已添加 Node.js {}", version), StatusLevel::Success);
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
        let version = self.new_java_version.clone();
        self.config
            .add_java_version(version.clone(), self.new_java_path.clone());
        if let Err(e) = self.save_config() {
            self.set_status(format!("保存配置失败：{}", e), StatusLevel::Error);
        } else {
            self.java_versions.push(VersionEntry {
                version: version.clone(),
                path: self.new_java_path.clone(),
            });
            self.new_java_version.clear();
            self.new_java_path.clear();
            self.show_add_java_dialog = false;
            self.set_status(format!("已添加 Java {}", version), StatusLevel::Success);
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
        let node_ver = if self.selected_node_for_mapping.is_empty() {
            None
        } else {
            Some(self.selected_node_for_mapping.clone())
        };
        let java_ver = if self.selected_java_for_mapping.is_empty() {
            None
        } else {
            Some(self.selected_java_for_mapping.clone())
        };

        self.config
            .add_path_mapping(self.new_mapping_path.clone(), node_ver, java_ver);
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
            self.set_status(
                format!("已添加路径映射：{}", self.new_mapping_path.clone()),
                StatusLevel::Success,
            );
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
            std::process::Command::new("explorer")
                .arg("/select,")
                .arg(&path)
                .spawn()
                .ok();
            self.set_status("已打开配置文件夹", StatusLevel::Success);
        } else {
            self.set_status("配置文件不存在", StatusLevel::Warning);
        }
    }

    fn browse_for_folder(&self, title: &str) -> Option<String> {
        rfd::FileDialog::new()
            .set_title(title)
            .pick_folder()
            .map(|p| p.to_string_lossy().to_string())
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::TabSelected(tab) => self.active_tab = tab,
            Message::InstallHook => self.install_hook(),
            Message::UninstallHook => self.uninstall_hook(),
            Message::OpenAddNodeDialog => self.show_add_node_dialog = true,
            Message::CloseAddNodeDialog => {
                self.show_add_node_dialog = false;
                self.new_node_version.clear();
                self.new_node_path.clear();
            }
            Message::NodeVersionInputChanged(value) => self.new_node_version = value,
            Message::NodePathInputChanged(value) => self.new_node_path = value,
            Message::BrowseNodePath => {
                if let Some(path) = self.browse_for_folder("选择 Node.js 安装目录") {
                    self.new_node_path = path;
                }
            }
            Message::AddNodeVersion => self.add_node_version(),
            Message::RemoveNodeVersion(version) => self.remove_node_version(&version),
            Message::OpenAddJavaDialog => self.show_add_java_dialog = true,
            Message::CloseAddJavaDialog => {
                self.show_add_java_dialog = false;
                self.new_java_version.clear();
                self.new_java_path.clear();
            }
            Message::JavaVersionInputChanged(value) => self.new_java_version = value,
            Message::JavaPathInputChanged(value) => self.new_java_path = value,
            Message::BrowseJavaPath => {
                if let Some(path) = self.browse_for_folder("选择 Java 安装目录") {
                    self.new_java_path = path;
                }
            }
            Message::JavaVersionInputForMappingChanged(value) => {
                self.selected_java_for_mapping = value
            }
            Message::AddJavaVersion => self.add_java_version(),
            Message::RemoveJavaVersion(version) => self.remove_java_version(&version),
            Message::OpenAddMappingDialog => self.show_add_mapping_dialog = true,
            Message::CloseAddMappingDialog => {
                self.show_add_mapping_dialog = false;
                self.new_mapping_path.clear();
                self.selected_node_for_mapping.clear();
                self.selected_java_for_mapping.clear();
            }
            Message::MappingPathInputChanged(value) => self.new_mapping_path = value,
            Message::BrowseMappingPath => {
                if let Some(path) = self.browse_for_folder("选择项目目录") {
                    self.new_mapping_path = path;
                }
            }
            Message::NodeVersionInputForMappingChanged(value) => {
                self.selected_node_for_mapping = value
            }
            Message::AddPathMapping => self.add_path_mapping(),
            Message::RemovePathMapping(path) => self.remove_path_mapping(&path),
            Message::RefreshConfig => self.refresh_config(),
            Message::OpenConfigFolder => self.open_config_folder(),
        }
        Task::none()
    }

    fn view(&self) -> Element<Message> {
        row![self.sidebar(), self.content()]
            .spacing(0)
            .height(Fill)
            .into()
    }

    fn sidebar(&self) -> Element<Message> {
        let logo = Text::new(Bootstrap::Plug.to_string())
            .font(BOOTSTRAP_FONT)
            .size(24)
            .color(FluentColors::TEXT_PRIMARY);
        let title = text("Env Switcher")
            .size(16)
            .color(FluentColors::TEXT_PRIMARY);
        let subtitle = text("多版本环境管理工具")
            .size(10)
            .color(FluentColors::TEXT_SECONDARY);

        let header_col = column![logo, title, subtitle]
            .spacing(4)
            .align_x(Alignment::Center)
            .width(Fill)
            .padding(20);

        let header_card =
            container(header_col)
                .width(Fill)
                .padding(16)
                .style(|_| container::Style {
                    background: Some(FluentColors::BG_CARD.into()),
                    border: Border {
                        color: FluentColors::BORDER_SUBTLE,
                        width: 1.0,
                        ..Default::default()
                    },
                    ..Default::default()
                });

        let mut nav_buttons: Vec<Element<Message>> = Vec::new();
        for tab in Tab::all() {
            let is_active = self.active_tab == *tab;
            let icon = Text::new(match tab {
                Tab::Overview => Bootstrap::BarChart.to_string(),
                Tab::Node => Bootstrap::Cpu.to_string(),
                Tab::Java => Bootstrap::Cup.to_string(),
                Tab::Paths => Bootstrap::Folder.to_string(),
            })
            .font(BOOTSTRAP_FONT)
            .size(16);
            let btn_content = row![
                icon.color(FluentColors::TEXT_PRIMARY),
                Space::with_width(10),
                text(tab.label()).size(14)
            ]
            .align_y(Alignment::Center);

            let btn = button(btn_content)
                .padding([12, 16])
                .width(Fill)
                .style(
                    move |_theme: &Theme, status: button::Status| button::Style {
                        background: Some(
                            if is_active || matches!(status, button::Status::Hovered) {
                                FluentColors::BG_CARD
                            } else {
                                Color::TRANSPARENT
                            }
                            .into(),
                        ),
                        border: Border {
                            color: if is_active {
                                FluentColors::SUCCESS
                            } else {
                                Color::TRANSPARENT
                            },
                            width: if is_active { 2.0 } else { 0.0 },
                            radius: 8.0.into(),
                            ..Default::default()
                        },
                        text_color: FluentColors::TEXT_PRIMARY,
                        ..Default::default()
                    },
                )
                .on_press(Message::TabSelected(*tab));

            nav_buttons.push(container(btn).width(Fill).padding([8, 12]).into());
        }

        let nav_col = Column::with_children(nav_buttons)
            .spacing(4)
            .padding([0, 16]);

        let refresh_btn = button(
            row![
                Text::new(Bootstrap::ArrowClockwise.to_string())
                    .font(BOOTSTRAP_FONT)
                    .size(14)
                    .color(FluentColors::INFO),
                text("刷新").size(12)
            ]
            .spacing(8)
            .align_y(Alignment::Center),
        )
        .padding([8, 12])
        .style(|_theme: &Theme, _status: button::Status| button::Style {
            background: Some(FluentColors::BG_CARD.into()),
            border: Border {
                color: FluentColors::INFO,
                width: 1.0,
                radius: 6.0.into(),
                ..Default::default()
            },
            text_color: FluentColors::TEXT_PRIMARY,
            ..Default::default()
        })
        .on_press(Message::RefreshConfig);

        let config_btn = button(
            row![
                Text::new(Bootstrap::Folder.to_string())
                    .font(BOOTSTRAP_FONT)
                    .size(14)
                    .color(FluentColors::WARNING),
                text("配置").size(12)
            ]
            .spacing(8)
            .align_y(Alignment::Center),
        )
        .padding([8, 12])
        .style(|_theme: &Theme, _status: button::Status| button::Style {
            background: Some(FluentColors::BG_CARD.into()),
            border: Border {
                color: FluentColors::WARNING,
                width: 1.0,
                radius: 6.0.into(),
                ..Default::default()
            },
            text_color: FluentColors::TEXT_PRIMARY,
            ..Default::default()
        })
        .on_press(Message::OpenConfigFolder);

        let install_btn = button(
            row![
                Text::new(Bootstrap::Folder.to_string())
                    .font(BOOTSTRAP_FONT)
                    .size(14)
                    .color(FluentColors::WARNING),
                text("重装").size(12)
            ]
            .spacing(8)
            .align_y(Alignment::Center),
        )
        .padding([8, 12])
        .style(|_theme: &Theme, _status: button::Status| button::Style {
            background: Some(FluentColors::BG_CARD.into()),
            border: Border {
                color: FluentColors::SUCCESS,
                width: 1.0,
                radius: 6.0.into(),
                ..Default::default()
            },
            text_color: FluentColors::TEXT_PRIMARY,
            ..Default::default()
        })
        .on_press(Message::InstallHook);

        let uninstall_btn = button(
            row![
                Text::new(Bootstrap::Folder.to_string())
                    .font(BOOTSTRAP_FONT)
                    .size(14)
                    .color(FluentColors::WARNING),
                text("卸载").size(12)
            ]
            .spacing(8)
            .align_y(Alignment::Center),
        )
        .padding([8, 12])
        .style(|_theme: &Theme, _status: button::Status| button::Style {
            background: Some(FluentColors::BG_CARD.into()),
            border: Border {
                color: FluentColors::ERROR,
                width: 1.0,
                radius: 6.0.into(),
                ..Default::default()
            },
            text_color: FluentColors::TEXT_PRIMARY,
            ..Default::default()
        })
        .on_press(Message::UninstallHook);

        let r1 = row![install_btn, uninstall_btn]
            .spacing(8)
            .align_y(Alignment::Center);
        let r2 = row![refresh_btn, config_btn]
            .spacing(8)
            .align_y(Alignment::Center);

        let bottom_btns = column![r1, r2].spacing(8).padding(16);

        let sidebar_col = column![
            header_card,
            Space::with_height(20),
            nav_col,
            container(Space::with_height(Fill)).height(Fill),
            bottom_btns
        ]
        .spacing(0);

        container(sidebar_col)
            .width(Length::Fixed(220.0))
            .height(Fill)
            .style(|_| container::Style {
                background: Some(FluentColors::BG_BASE.into()),
                border: Border {
                    color: FluentColors::BORDER_DEFAULT,
                    width: 1.0,
                    ..Default::default()
                },
                ..Default::default()
            })
            .into()
    }

    fn content(&self) -> Element<Message> {
        let content = match self.active_tab {
            Tab::Overview => self.overview_view(),
            Tab::Node => self.node_view(),
            Tab::Java => self.java_view(),
            Tab::Paths => self.paths_view(),
        };

        let status_text = if let Some(ref status) = self.status_message {
            let color = match status.level {
                StatusLevel::Info => FluentColors::INFO,
                StatusLevel::Success => FluentColors::SUCCESS,
                StatusLevel::Warning => FluentColors::WARNING,
                StatusLevel::Error => FluentColors::ERROR,
            };
            text(&status.text).color(color).size(12)
        } else {
            text("✓ 就绪").color(FluentColors::SUCCESS).size(12)
        };

        let status_bar = container(status_text)
            .padding([8, 16])
            .width(Fill)
            .style(|_| container::Style {
                background: Some(FluentColors::BG_CARD.into()),
                border: Border {
                    color: FluentColors::BORDER_SUBTLE,
                    width: 1.0,
                    ..Default::default()
                },
                ..Default::default()
            });

        column![
            container(content).padding(24).height(Fill).width(Fill),
            status_bar
        ]
        .spacing(0)
        .into()
    }

    fn overview_view(&self) -> Element<Message> {
        let hook_card = self.hook_status_card();
        let node_card = self.node_summary_card();
        let java_card = self.java_summary_card();
        let cards_row = row![hook_card, node_card, java_card].spacing(16);
        let current_env_card = self.current_env_card();
        scrollable(column![cards_row, current_env_card].spacing(24)).into()
    }

    fn hook_status_card(&self) -> Element<Message> {
        let (status_icon, status_color, status_text_val) = if self.hook_installed {
            ("●", FluentColors::SUCCESS, "已安装")
        } else {
            ("○", FluentColors::TEXT_MUTED, "未安装")
        };
        let status_row = row![
            text(status_icon).color(status_color).size(16),
            text(status_text_val).size(14)
        ]
        .spacing(8);

        let card_content: Element<Message> = column![
            row![
                Text::new(Bootstrap::Plug.to_string())
                    .font(BOOTSTRAP_FONT)
                    .size(20)
                    .color(FluentColors::TEXT_PRIMARY),
                text("Hook 状态").size(14).color(FluentColors::TEXT_PRIMARY)
            ]
            .spacing(8),
            Space::with_height(12),
            status_row,
            Space::with_height(16),
            // action_row
        ]
        .spacing(8)
        .into();

        self.card(card_content, 200.0, 140.0)
    }

    fn node_summary_card(&self) -> Element<Message> {
        let node_count = self.node_versions.len();
        let project_count = self
            .path_mappings
            .iter()
            .filter(|m| !m.node_version.is_empty())
            .count();

        let card_content: Element<Message> = column![
            row![
                Text::new(Bootstrap::Cpu.to_string())
                    .font(BOOTSTRAP_FONT)
                    .size(20)
                    .color(FluentColors::NODE_GREEN),
                text("Node.js").size(14).color(FluentColors::TEXT_PRIMARY)
            ]
            .spacing(8),
            Space::with_height(12),
            text(format!("{} 个版本", node_count))
                .size(28)
                .color(FluentColors::TEXT_PRIMARY),
            text(format!("{} 个项目", project_count))
                .size(12)
                .color(FluentColors::TEXT_SECONDARY),
            Space::with_height(16),
            button(text("管理 →").size(12))
                .padding([6, 12])
                .style(|_theme: &Theme, _status: button::Status| {
                    button::Style {
                        background: Some(Color::TRANSPARENT.into()),
                        border: Border {
                            color: FluentColors::NODE_GREEN,
                            width: 1.0,
                            radius: 6.0.into(),
                            ..Default::default()
                        },
                        text_color: FluentColors::TEXT_PRIMARY,
                        ..Default::default()
                    }
                })
                .on_press(Message::TabSelected(Tab::Node))
        ]
        .spacing(8)
        .into();

        self.card(card_content, 200.0, 140.0)
    }

    fn java_summary_card(&self) -> Element<Message> {
        let java_count = self.java_versions.len();
        let project_count = self
            .path_mappings
            .iter()
            .filter(|m| !m.java_version.is_empty())
            .count();

        let card_content: Element<Message> = column![
            row![
                Text::new(Bootstrap::Cup.to_string())
                    .font(BOOTSTRAP_FONT)
                    .size(20)
                    .color(FluentColors::JAVA_RED),
                text("Java").size(14).color(FluentColors::TEXT_PRIMARY)
            ]
            .spacing(8),
            Space::with_height(12),
            text(format!("{} 个版本", java_count))
                .size(28)
                .color(FluentColors::TEXT_PRIMARY),
            text(format!("{} 个项目", project_count))
                .size(12)
                .color(FluentColors::TEXT_SECONDARY),
            Space::with_height(16),
            button(text("管理 →").size(12))
                .padding([6, 12])
                .style(|_theme: &Theme, _status: button::Status| {
                    button::Style {
                        background: Some(Color::TRANSPARENT.into()),
                        border: Border {
                            color: FluentColors::JAVA_RED,
                            width: 1.0,
                            radius: 6.0.into(),
                            ..Default::default()
                        },
                        text_color: FluentColors::TEXT_PRIMARY,
                        ..Default::default()
                    }
                })
                .on_press(Message::TabSelected(Tab::Java))
        ]
        .spacing(8)
        .into();

        self.card(card_content, 200.0, 140.0)
    }

    fn current_env_card(&self) -> Element<Message> {
        let node_label = if let Some(ref node) = self.current_status.node_version {
            let icon = Text::new(Bootstrap::Cpu.to_string())
                .font(BOOTSTRAP_FONT)
                .size(14)
                .color(FluentColors::NODE_GREEN);
            row![
                icon,
                text(format!("Node: v{}", node))
                    .color(FluentColors::NODE_GREEN)
                    .size(14)
            ]
            .spacing(4)
        } else {
            let icon = Text::new(Bootstrap::Cpu.to_string())
                .font(BOOTSTRAP_FONT)
                .size(14)
                .color(FluentColors::TEXT_MUTED);
            row![
                icon,
                text("Node: 未配置")
                    .color(FluentColors::TEXT_MUTED)
                    .size(14)
            ]
            .spacing(4)
        };

        let java_label = if let Some(ref java) = self.current_status.java_version {
            let icon = Text::new(Bootstrap::Cup.to_string())
                .font(BOOTSTRAP_FONT)
                .size(14)
                .color(FluentColors::JAVA_RED);
            row![
                icon,
                text(format!("Java: v{}", java))
                    .color(FluentColors::JAVA_RED)
                    .size(14)
            ]
            .spacing(4)
        } else {
            let icon = Text::new(Bootstrap::Cup.to_string())
                .font(BOOTSTRAP_FONT)
                .size(14)
                .color(FluentColors::TEXT_MUTED);
            row![
                icon,
                text("Java: 未配置")
                    .color(FluentColors::TEXT_MUTED)
                    .size(14)
            ]
            .spacing(4)
        };

        let status_indicator = if self.current_status.matched_path.is_some() {
            text("● 已激活").color(FluentColors::SUCCESS).size(12)
        } else {
            text("").size(12)
        };

        let card_content: Element<Message> = column![
            row![
                Text::new(Bootstrap::Folder.to_string())
                    .font(BOOTSTRAP_FONT)
                    .size(18)
                    .color(FluentColors::TEXT_PRIMARY),
                text("当前路径").size(14).color(FluentColors::TEXT_PRIMARY),
                Space::with_width(8),
                text(&self.current_path)
                    .color(FluentColors::TEXT_SECONDARY)
                    .size(12),
                horizontal_space(),
                status_indicator
            ]
            .align_y(Alignment::Center),
            Space::with_height(10),
            row![node_label, Space::with_width(30), java_label].spacing(30)
        ]
        .spacing(8)
        .into();

        container(card_content)
            .padding(16)
            .width(Fill)
            .style(|_| container::Style {
                background: Some(FluentColors::BG_CARD.into()),
                border: Border {
                    color: FluentColors::BORDER_DEFAULT,
                    width: 1.0,
                    radius: 12.0.into(),
                    ..Default::default()
                },
                ..Default::default()
            })
            .into()
    }

    fn node_view(&self) -> Element<Message> {
        let add_btn = button(text("+ 添加版本").size(12))
            .padding([6, 12])
            .style(|_theme: &Theme, _status: button::Status| button::Style {
                background: Some(Color::TRANSPARENT.into()),
                border: Border {
                    color: FluentColors::SUCCESS,
                    width: 1.0,
                    radius: 6.0.into(),
                    ..Default::default()
                },
                text_color: FluentColors::TEXT_PRIMARY,
                ..Default::default()
            })
            .on_press(Message::OpenAddNodeDialog);

        let header = row![
            Text::new(Bootstrap::Cpu.to_string())
                .font(BOOTSTRAP_FONT)
                .size(20)
                .color(FluentColors::NODE_GREEN),
            text("Node.js 版本")
                .size(16)
                .color(FluentColors::TEXT_PRIMARY),
            horizontal_space(),
            add_btn
        ]
        .align_y(Alignment::Center)
        .spacing(16);

        let mut versions: Vec<Element<Message>> = Vec::new();
        for entry in &self.node_versions {
            let version_card: Element<Message> = row![
                column![
                    text(format!("v{}", entry.version))
                        .size(15)
                        .color(FluentColors::NODE_GREEN),
                    text(&entry.path)
                        .size(11)
                        .color(FluentColors::TEXT_SECONDARY)
                ]
                .spacing(4),
                horizontal_space(),
                button(
                    Text::new(Bootstrap::Trash.to_string())
                        .font(BOOTSTRAP_FONT)
                        .size(14)
                )
                .padding([6, 10])
                .style(|_theme: &Theme, status: button::Status| {
                    let bg = if matches!(status, button::Status::Hovered) {
                        FluentColors::ERROR
                    } else {
                        Color::TRANSPARENT
                    };
                    button::Style {
                        background: Some(bg.into()),
                        border: Border {
                            color: FluentColors::ERROR,
                            width: 1.0,
                            radius: 6.0.into(),
                            ..Default::default()
                        },
                        text_color: if matches!(status, button::Status::Hovered) {
                            Color::WHITE
                        } else {
                            FluentColors::ERROR
                        },
                        ..Default::default()
                    }
                })
                .on_press(Message::RemoveNodeVersion(entry.version.clone()))
            ]
            .align_y(Alignment::Center)
            .spacing(16)
            .into();

            versions.push(self.card_row(version_card).into());
            versions.push(Space::with_height(8).into());
        }

        scrollable(
            column![
                header,
                Space::with_height(12),
                Column::with_children(versions)
            ]
            .spacing(0),
        )
        .into()
    }

    fn java_view(&self) -> Element<Message> {
        let add_btn = button(text("+ 添加版本").size(12))
            .padding([6, 12])
            .style(|_theme: &Theme, _status: button::Status| button::Style {
                background: Some(Color::TRANSPARENT.into()),
                border: Border {
                    color: FluentColors::SUCCESS,
                    width: 1.0,
                    radius: 6.0.into(),
                    ..Default::default()
                },
                text_color: FluentColors::TEXT_PRIMARY,
                ..Default::default()
            })
            .on_press(Message::OpenAddJavaDialog);

        let header = row![
            Text::new(Bootstrap::Cup.to_string())
                .font(BOOTSTRAP_FONT)
                .size(20)
                .color(FluentColors::JAVA_RED),
            text("Java 版本").size(16).color(FluentColors::TEXT_PRIMARY),
            horizontal_space(),
            add_btn
        ]
        .align_y(Alignment::Center)
        .spacing(16);

        let mut versions: Vec<Element<Message>> = Vec::new();
        for entry in &self.java_versions {
            let version_card: Element<Message> = row![
                column![
                    text(format!("v{}", entry.version))
                        .size(15)
                        .color(FluentColors::JAVA_RED),
                    text(&entry.path)
                        .size(11)
                        .color(FluentColors::TEXT_SECONDARY)
                ]
                .spacing(4),
                horizontal_space(),
                button(
                    Text::new(Bootstrap::Trash.to_string())
                        .font(BOOTSTRAP_FONT)
                        .size(14)
                )
                .padding([6, 10])
                .style(|_theme: &Theme, status: button::Status| {
                    let bg = if matches!(status, button::Status::Hovered) {
                        FluentColors::ERROR
                    } else {
                        Color::TRANSPARENT
                    };
                    button::Style {
                        background: Some(bg.into()),
                        border: Border {
                            color: FluentColors::ERROR,
                            width: 1.0,
                            radius: 6.0.into(),
                            ..Default::default()
                        },
                        text_color: if matches!(status, button::Status::Hovered) {
                            Color::WHITE
                        } else {
                            FluentColors::ERROR
                        },
                        ..Default::default()
                    }
                })
                .on_press(Message::RemoveJavaVersion(entry.version.clone()))
            ]
            .align_y(Alignment::Center)
            .spacing(16)
            .into();

            versions.push(self.card_row(version_card).into());
            versions.push(Space::with_height(8).into());
        }

        scrollable(
            column![
                header,
                Space::with_height(12),
                Column::with_children(versions)
            ]
            .spacing(0),
        )
        .into()
    }

    fn paths_view(&self) -> Element<Message> {
        let add_btn = button(text("+ 添加映射").size(12))
            .padding([6, 12])
            .style(|_theme: &Theme, _status: button::Status| button::Style {
                background: Some(Color::TRANSPARENT.into()),
                border: Border {
                    color: FluentColors::SUCCESS,
                    width: 1.0,
                    radius: 6.0.into(),
                    ..Default::default()
                },
                text_color: FluentColors::TEXT_PRIMARY,
                ..Default::default()
            })
            .on_press(Message::OpenAddMappingDialog);

        let header = row![
            Text::new(Bootstrap::Folder.to_string())
                .font(BOOTSTRAP_FONT)
                .size(20)
                .color(FluentColors::TEXT_PRIMARY),
            text("路径映射").size(16).color(FluentColors::TEXT_PRIMARY),
            horizontal_space(),
            add_btn
        ]
        .align_y(Alignment::Center)
        .spacing(16);

        let mut mappings: Vec<Element<Message>> = Vec::new();
        for entry in &self.path_mappings {
            let node_label = if !entry.node_version.is_empty() {
                let icon = Text::new(Bootstrap::Cpu.to_string())
                    .font(BOOTSTRAP_FONT)
                    .size(12)
                    .color(FluentColors::NODE_GREEN);
                row![
                    icon,
                    text(entry.node_version.clone())
                        .color(FluentColors::NODE_GREEN)
                        .size(12)
                ]
                .spacing(4)
            } else {
                let icon = Text::new(Bootstrap::Cpu.to_string())
                    .font(BOOTSTRAP_FONT)
                    .size(12)
                    .color(FluentColors::TEXT_MUTED);
                row![
                    icon,
                    text("未配置").color(FluentColors::TEXT_MUTED).size(12)
                ]
                .spacing(4)
            };

            let java_label = if !entry.java_version.is_empty() {
                let icon = Text::new(Bootstrap::Cup.to_string())
                    .font(BOOTSTRAP_FONT)
                    .size(12)
                    .color(FluentColors::JAVA_RED);
                row![
                    icon,
                    text(entry.java_version.clone())
                        .color(FluentColors::JAVA_RED)
                        .size(12)
                ]
                .spacing(4)
            } else {
                let icon = Text::new(Bootstrap::Cup.to_string())
                    .font(BOOTSTRAP_FONT)
                    .size(12)
                    .color(FluentColors::TEXT_MUTED);
                row![
                    icon,
                    text("未配置").color(FluentColors::TEXT_MUTED).size(12)
                ]
                .spacing(4)
            };

            let mapping_card: Element<Message> = row![
                column![
                    row![
                        Text::new(Bootstrap::Folder.to_string())
                            .font(BOOTSTRAP_FONT)
                            .size(13)
                            .color(FluentColors::TEXT_PRIMARY),
                        text(&entry.path).size(13).color(FluentColors::TEXT_PRIMARY)
                    ]
                    .spacing(4),
                    text(&entry.path)
                        .size(11)
                        .color(FluentColors::TEXT_SECONDARY),
                    Space::with_height(6),
                    row![node_label, Space::with_width(15), java_label].spacing(15)
                ]
                .spacing(4),
                horizontal_space(),
                button(
                    Text::new(Bootstrap::Trash.to_string())
                        .font(BOOTSTRAP_FONT)
                        .size(14)
                )
                .padding([6, 10])
                .style(|_theme: &Theme, status: button::Status| {
                    let bg = if matches!(status, button::Status::Hovered) {
                        FluentColors::ERROR
                    } else {
                        Color::TRANSPARENT
                    };
                    button::Style {
                        background: Some(bg.into()),
                        border: Border {
                            color: FluentColors::ERROR,
                            width: 1.0,
                            radius: 6.0.into(),
                            ..Default::default()
                        },
                        text_color: if matches!(status, button::Status::Hovered) {
                            Color::WHITE
                        } else {
                            FluentColors::ERROR
                        },
                        ..Default::default()
                    }
                })
                .on_press(Message::RemovePathMapping(entry.path.clone()))
            ]
            .align_y(Alignment::Center)
            .spacing(16)
            .into();

            mappings.push(self.card_row(mapping_card).into());
            mappings.push(Space::with_height(8).into());
        }

        scrollable(
            column![
                header,
                Space::with_height(12),
                Column::with_children(mappings)
            ]
            .spacing(0),
        )
        .into()
    }

    fn card<'a>(
        &self,
        content: Element<'a, Message>,
        width: f32,
        height: f32,
    ) -> Element<'a, Message> {
        container(content)
            .padding(16)
            .width(width)
            .height(height)
            .style(|_| container::Style {
                background: Some(FluentColors::BG_CARD.into()),
                border: Border {
                    color: FluentColors::BORDER_DEFAULT,
                    width: 1.0,
                    radius: 12.0.into(),
                    ..Default::default()
                },
                ..Default::default()
            })
            .into()
    }

    fn card_row<'a>(&self, content: Element<'a, Message>) -> Element<'a, Message> {
        container(content)
            .padding(16)
            .width(Fill)
            .style(|_| container::Style {
                background: Some(FluentColors::BG_CARD.into()),
                border: Border {
                    color: FluentColors::BORDER_DEFAULT,
                    width: 1.0,
                    radius: 12.0.into(),
                    ..Default::default()
                },
                ..Default::default()
            })
            .into()
    }

    fn add_node_dialog(&self) -> Element<Message> {
        let version_input = text_input("版本号，如 18.0.0", &self.new_node_version)
            .padding(8)
            .on_input(Message::NodeVersionInputChanged);
        let path_input = text_input("安装路径", &self.new_node_path)
            .padding(8)
            .on_input(Message::NodePathInputChanged);
        let browse_btn = button(text("浏览...").size(12))
            .padding([6, 12])
            .style(|_theme: &Theme, _status: button::Status| button::Style {
                background: Some(Color::TRANSPARENT.into()),
                border: Border {
                    color: FluentColors::INFO,
                    width: 1.0,
                    radius: 6.0.into(),
                    ..Default::default()
                },
                text_color: FluentColors::TEXT_PRIMARY,
                ..Default::default()
            })
            .on_press(Message::BrowseNodePath);
        let path_row = row![path_input, browse_btn].spacing(8);
        let button_row = row![
            button(text("确定").size(12))
                .padding([8, 24])
                .style(|_theme: &Theme, _status: button::Status| {
                    button::Style {
                        background: Some(FluentColors::BG_BUTTON_PRIMARY.into()),
                        border: Border {
                            color: FluentColors::SUCCESS,
                            width: 1.0,
                            radius: 6.0.into(),
                            ..Default::default()
                        },
                        text_color: Color::WHITE,
                        ..Default::default()
                    }
                })
                .on_press(Message::AddNodeVersion),
            button(text("取消").size(12))
                .padding([8, 24])
                .style(|_theme: &Theme, _status: button::Status| {
                    button::Style {
                        background: Some(Color::TRANSPARENT.into()),
                        border: Border {
                            color: FluentColors::BORDER_DEFAULT,
                            width: 1.0,
                            radius: 6.0.into(),
                            ..Default::default()
                        },
                        text_color: FluentColors::TEXT_PRIMARY,
                        ..Default::default()
                    }
                })
                .on_press(Message::CloseAddNodeDialog)
        ]
        .spacing(16);
        column![
            text("添加 Node.js 版本")
                .size(16)
                .color(FluentColors::TEXT_PRIMARY),
            Space::with_height(16),
            text("版本号:").size(12).color(FluentColors::TEXT_SECONDARY),
            Space::with_height(4),
            version_input,
            Space::with_height(12),
            text("安装路径:")
                .size(12)
                .color(FluentColors::TEXT_SECONDARY),
            Space::with_height(4),
            path_row,
            Space::with_height(20),
            button_row
        ]
        .spacing(8)
        .into()
    }

    fn add_java_dialog(&self) -> Element<Message> {
        let version_input = text_input("版本号，如 17", &self.new_java_version)
            .padding(8)
            .on_input(Message::JavaVersionInputChanged);
        let path_input = text_input("安装路径", &self.new_java_path)
            .padding(8)
            .on_input(Message::JavaPathInputChanged);
        let browse_btn = button(text("浏览...").size(12))
            .padding([6, 12])
            .style(|_theme: &Theme, _status: button::Status| button::Style {
                background: Some(Color::TRANSPARENT.into()),
                border: Border {
                    color: FluentColors::INFO,
                    width: 1.0,
                    radius: 6.0.into(),
                    ..Default::default()
                },
                text_color: FluentColors::TEXT_PRIMARY,
                ..Default::default()
            })
            .on_press(Message::BrowseJavaPath);
        let path_row = row![path_input, browse_btn].spacing(8);
        let button_row = row![
            button(text("确定").size(12))
                .padding([8, 24])
                .style(|_theme: &Theme, _status: button::Status| {
                    button::Style {
                        background: Some(FluentColors::BG_BUTTON_PRIMARY.into()),
                        border: Border {
                            color: FluentColors::SUCCESS,
                            width: 1.0,
                            radius: 6.0.into(),
                            ..Default::default()
                        },
                        text_color: Color::WHITE,
                        ..Default::default()
                    }
                })
                .on_press(Message::AddJavaVersion),
            button(text("取消").size(12))
                .padding([8, 24])
                .style(|_theme: &Theme, _status: button::Status| {
                    button::Style {
                        background: Some(Color::TRANSPARENT.into()),
                        border: Border {
                            color: FluentColors::BORDER_DEFAULT,
                            width: 1.0,
                            radius: 6.0.into(),
                            ..Default::default()
                        },
                        text_color: FluentColors::TEXT_PRIMARY,
                        ..Default::default()
                    }
                })
                .on_press(Message::CloseAddJavaDialog)
        ]
        .spacing(16);
        column![
            text("添加 Java 版本")
                .size(16)
                .color(FluentColors::TEXT_PRIMARY),
            Space::with_height(16),
            text("版本号:").size(12).color(FluentColors::TEXT_SECONDARY),
            Space::with_height(4),
            version_input,
            Space::with_height(12),
            text("安装路径:")
                .size(12)
                .color(FluentColors::TEXT_SECONDARY),
            Space::with_height(4),
            path_row,
            Space::with_height(20),
            button_row
        ]
        .spacing(8)
        .into()
    }

    fn add_mapping_dialog(&self) -> Element<Message> {
        let path_input = text_input("项目路径", &self.new_mapping_path)
            .padding(8)
            .on_input(Message::MappingPathInputChanged);
        let browse_btn = button(text("浏览...").size(12))
            .padding([6, 12])
            .style(|_theme: &Theme, _status: button::Status| button::Style {
                background: Some(Color::TRANSPARENT.into()),
                border: Border {
                    color: FluentColors::INFO,
                    width: 1.0,
                    radius: 6.0.into(),
                    ..Default::default()
                },
                text_color: FluentColors::TEXT_PRIMARY,
                ..Default::default()
            })
            .on_press(Message::BrowseMappingPath);
        let path_row = row![path_input, browse_btn].spacing(8);

        // Use text inputs instead of picklists to avoid lifetime issues
        let node_input = text_input("Node.js 版本号（可选）", &self.selected_node_for_mapping)
            .padding(8)
            .on_input(Message::NodeVersionInputForMappingChanged);
        let java_input = text_input("Java 版本号（可选）", &self.selected_java_for_mapping)
            .padding(8)
            .on_input(Message::JavaVersionInputForMappingChanged);

        let button_row = row![
            button(text("确定").size(12))
                .padding([8, 24])
                .style(|_theme: &Theme, _status: button::Status| {
                    button::Style {
                        background: Some(FluentColors::BG_BUTTON_PRIMARY.into()),
                        border: Border {
                            color: FluentColors::SUCCESS,
                            width: 1.0,
                            radius: 6.0.into(),
                            ..Default::default()
                        },
                        text_color: Color::WHITE,
                        ..Default::default()
                    }
                })
                .on_press(Message::AddPathMapping),
            button(text("取消").size(12))
                .padding([8, 24])
                .style(|_theme: &Theme, _status: button::Status| {
                    button::Style {
                        background: Some(Color::TRANSPARENT.into()),
                        border: Border {
                            color: FluentColors::BORDER_DEFAULT,
                            width: 1.0,
                            radius: 6.0.into(),
                            ..Default::default()
                        },
                        text_color: FluentColors::TEXT_PRIMARY,
                        ..Default::default()
                    }
                })
                .on_press(Message::CloseAddMappingDialog)
        ]
        .spacing(16);
        column![
            text("添加路径映射")
                .size(16)
                .color(FluentColors::TEXT_PRIMARY),
            Space::with_height(16),
            text("项目路径:")
                .size(12)
                .color(FluentColors::TEXT_SECONDARY),
            Space::with_height(4),
            path_row,
            Space::with_height(12),
            text("Node.js 版本:")
                .size(12)
                .color(FluentColors::TEXT_SECONDARY),
            Space::with_height(4),
            node_input,
            Space::with_height(12),
            text("Java 版本:")
                .size(12)
                .color(FluentColors::TEXT_SECONDARY),
            Space::with_height(4),
            java_input,
            Space::with_height(20),
            button_row
        ]
        .spacing(8)
        .into()
    }
}

fn update(state: &mut EnvSwitcherApp, message: Message) -> Task<Message> {
    state.update(message)
}

fn view(state: &EnvSwitcherApp) -> Element<Message> {
    let main_view = state.view();

    if state.show_add_node_dialog {
        let dialog = state.add_node_dialog();
        let overlay = container(dialog)
            .width(400.0)
            .height(320.0)
            .style(|_| container::Style {
                background: Some(FluentColors::BG_CARD.into()),
                border: Border {
                    color: FluentColors::BORDER_DEFAULT,
                    width: 1.0,
                    radius: 12.0.into(),
                    ..Default::default()
                },
                ..Default::default()
            });
        return container(column![
            container(main_view).width(Fill).height(Fill),
            container(overlay).center_x(Fill).center_y(Fill)
        ])
        .width(Fill)
        .height(Fill)
        .into();
    }

    if state.show_add_java_dialog {
        let dialog = state.add_java_dialog();
        let overlay = container(dialog)
            .width(400.0)
            .height(320.0)
            .style(|_| container::Style {
                background: Some(FluentColors::BG_CARD.into()),
                border: Border {
                    color: FluentColors::BORDER_DEFAULT,
                    width: 1.0,
                    radius: 12.0.into(),
                    ..Default::default()
                },
                ..Default::default()
            });
        return container(column![
            container(main_view).width(Fill).height(Fill),
            container(overlay).center_x(Fill).center_y(Fill)
        ])
        .width(Fill)
        .height(Fill)
        .into();
    }

    if state.show_add_mapping_dialog {
        let dialog = state.add_mapping_dialog();
        let overlay = container(dialog)
            .width(400.0)
            .height(380.0)
            .style(|_| container::Style {
                background: Some(FluentColors::BG_CARD.into()),
                border: Border {
                    color: FluentColors::BORDER_DEFAULT,
                    width: 1.0,
                    radius: 12.0.into(),
                    ..Default::default()
                },
                ..Default::default()
            });
        return container(column![
            container(main_view).width(Fill).height(Fill),
            container(overlay).center_x(Fill).center_y(Fill)
        ])
        .width(Fill)
        .height(Fill)
        .into();
    }

    main_view
}

/// 运行 GUI 应用
pub fn run_gui() -> Result<()> {
    // 加载中文字体（Windows 系统黑体）
    let chinese_font = include_bytes!("C:/Windows/Fonts/simhei.ttf");

    // 加载 Bootstrap Icons 字体
    let bootstrap_font = iced_aw::BOOTSTRAP_FONT_BYTES;

    iced::application("Env Switcher", update, view)
        .window(window::Settings {
            size: iced::Size::new(1100.0, 750.0),
            min_size: Some(iced::Size::new(900.0, 600.0)),
            ..Default::default()
        })
        .theme(|_| Theme::Dark)
        .default_font(iced::Font::with_name("SimHei"))
        .font(chinese_font)
        .font(bootstrap_font)
        .run_with(EnvSwitcherApp::new)
        .map_err(|e| anyhow::anyhow!("{}", e))?;

    Ok(())
}
