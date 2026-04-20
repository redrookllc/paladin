use egui::{Color32, RichText, Stroke};
use egui_plot::{BoxElem, BoxPlot, BoxSpread, HLine, Legend, Line, Plot, PlotPoints};
use paladin_ai::{ChatMessage, Client};
use paladin_core::candle::{Candle, Interval};
use paladin_core::config::Config;
use paladin_core::{FeatureBundle, TradeSignal};
use std::sync::mpsc::{channel, Receiver, Sender};

const ACCENT: Color32 = Color32::from_rgb(0x94, 0x11, 0x07);
const GREEN: Color32 = Color32::from_rgb(0x3a, 0xc2, 0x7a);
const RED: Color32 = Color32::from_rgb(0xe2, 0x4f, 0x4f);

#[derive(PartialEq)]
enum Tab {
    Chart,
    Signal,
    Chat,
    Settings,
}

pub struct PaladinApp {
    tab: Tab,
    config: Config,
    symbol: String,
    interval: Interval,
    candles: Vec<Candle>,
    signal: Option<TradeSignal>,
    chat_input: String,
    chat_log: Vec<(String, String)>,
    status: String,
    runtime: tokio::runtime::Runtime,
    events_rx: Receiver<AppEvent>,
    events_tx: Sender<AppEvent>,
}

enum AppEvent {
    Candles(Vec<Candle>),
    Signal(TradeSignal),
    ChatDelta(String),
    ChatDone,
    Error(String),
}

impl PaladinApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let mut visuals = egui::Visuals::dark();
        visuals.panel_fill = Color32::from_rgb(10, 10, 10);
        visuals.window_fill = Color32::from_rgb(18, 18, 18);
        visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0, Color32::from_gray(200));
        visuals.selection.bg_fill = ACCENT;
        cc.egui_ctx.set_visuals(visuals);
        let mut style = (*cc.egui_ctx.style()).clone();
        style.spacing.item_spacing = egui::vec2(8.0, 6.0);
        cc.egui_ctx.set_style(style);

        let config = Config::load().unwrap_or_default();
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .worker_threads(2)
            .build()
            .expect("tokio runtime");
        let (tx, rx) = channel();
        Self {
            tab: Tab::Chart,
            symbol: config.default_symbol.clone(),
            interval: Interval::parse(&config.default_interval).unwrap_or(Interval::OneDay),
            config,
            candles: Vec::new(),
            signal: None,
            chat_input: String::new(),
            chat_log: Vec::new(),
            status: "ready".into(),
            runtime,
            events_rx: rx,
            events_tx: tx,
        }
    }

    fn drain_events(&mut self, ctx: &egui::Context) {
        while let Ok(ev) = self.events_rx.try_recv() {
            match ev {
                AppEvent::Candles(c) => {
                    self.status = format!("loaded {} candles", c.len());
                    self.candles = c;
                }
                AppEvent::Signal(s) => {
                    self.status = format!("signal: {} {}", s.direction, s.symbol);
                    self.signal = Some(s);
                }
                AppEvent::ChatDelta(text) => {
                    if let Some(last) = self.chat_log.last_mut() {
                        last.1.push_str(&text);
                    }
                }
                AppEvent::ChatDone => {
                    self.status = "chat: done".into();
                }
                AppEvent::Error(e) => {
                    self.status = format!("error: {e}");
                }
            }
        }
        ctx.request_repaint_after(std::time::Duration::from_millis(50));
    }

    fn fetch_candles(&self) {
        let tx = self.events_tx.clone();
        let symbol = self.symbol.clone();
        let interval = self.interval;
        self.runtime.spawn(async move {
            match paladin_data::fetch_cached(&symbol, interval, None).await {
                Ok(c) => {
                    let _ = tx.send(AppEvent::Candles(c));
                }
                Err(e) => {
                    let _ = tx.send(AppEvent::Error(e.to_string()));
                }
            }
        });
    }

    fn request_signal(&self) {
        let tx = self.events_tx.clone();
        let symbol = self.symbol.clone();
        let interval = self.interval;
        let cfg = self.config.clone();
        self.runtime.spawn(async move {
            let candles = match paladin_data::fetch_cached(&symbol, interval, None).await {
                Ok(c) => c,
                Err(e) => {
                    let _ = tx.send(AppEvent::Error(e.to_string()));
                    return;
                }
            };
            let features = FeatureBundle::from_candles(&symbol, interval.as_yahoo(), &candles);
            let key = cfg.effective_api_key();
            let client = match Client::new(cfg.api_base_url, key, cfg.model) {
                Ok(c) => c,
                Err(e) => {
                    let _ = tx.send(AppEvent::Error(e.to_string()));
                    return;
                }
            };
            match client.signal(&features).await {
                Ok(s) => {
                    let _ = tx.send(AppEvent::Signal(s));
                }
                Err(e) => {
                    let _ = tx.send(AppEvent::Error(e.to_string()));
                }
            }
        });
    }

    fn send_chat(&mut self) {
        let user = std::mem::take(&mut self.chat_input);
        if user.trim().is_empty() {
            return;
        }
        self.chat_log.push((user.clone(), String::new()));
        let tx = self.events_tx.clone();
        let cfg = self.config.clone();
        let history = self.chat_log.clone();
        let symbol = self.symbol.clone();
        self.runtime.spawn(async move {
            let key = cfg.effective_api_key();
            let client = match Client::new(cfg.api_base_url, key, cfg.model) {
                Ok(c) => c,
                Err(e) => {
                    let _ = tx.send(AppEvent::Error(e.to_string()));
                    return;
                }
            };
            let mut messages = vec![ChatMessage::system(paladin_ai::CHAT_SYSTEM_PROMPT)];
            messages.push(ChatMessage::system(format!("User context symbol: {symbol}")));
            for (u, a) in history.iter().take(history.len().saturating_sub(1)) {
                messages.push(ChatMessage::user(u.clone()));
                if !a.is_empty() {
                    messages.push(ChatMessage::assistant(a.clone()));
                }
            }
            messages.push(ChatMessage::user(user));
            use futures::StreamExt;
            match client.chat_stream(&messages).await {
                Ok(mut stream) => {
                    while let Some(chunk) = stream.next().await {
                        match chunk {
                            Ok(text) => {
                                let _ = tx.send(AppEvent::ChatDelta(text));
                            }
                            Err(e) => {
                                let _ = tx.send(AppEvent::Error(e.to_string()));
                                break;
                            }
                        }
                    }
                    let _ = tx.send(AppEvent::ChatDone);
                }
                Err(e) => {
                    let _ = tx.send(AppEvent::Error(e.to_string()));
                }
            }
        });
    }

    fn ui_top(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label(RichText::new("PALADIN").color(ACCENT).strong().size(18.0));
            ui.separator();
            ui.selectable_value(&mut self.tab, Tab::Chart, "Chart");
            ui.selectable_value(&mut self.tab, Tab::Signal, "Signal");
            ui.selectable_value(&mut self.tab, Tab::Chat, "Chat");
            ui.selectable_value(&mut self.tab, Tab::Settings, "Settings");
            ui.separator();
            ui.label(RichText::new(&self.status).weak());
        });
    }

    fn ui_chart(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label("Symbol");
            let resp = ui.text_edit_singleline(&mut self.symbol);
            let iv_label = self.interval.as_yahoo();
            egui::ComboBox::from_label("Interval")
                .selected_text(iv_label)
                .show_ui(ui, |ui| {
                    for iv in [
                        Interval::OneMinute,
                        Interval::FiveMinute,
                        Interval::FifteenMinute,
                        Interval::OneHour,
                        Interval::OneDay,
                        Interval::OneWeek,
                    ] {
                        ui.selectable_value(&mut self.interval, iv, iv.as_yahoo());
                    }
                });
            if ui.button("Fetch").clicked() || resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                self.fetch_candles();
            }
        });

        if self.candles.is_empty() {
            ui.label(RichText::new("No candles loaded. Click Fetch.").weak());
            return;
        }

        let boxes: Vec<BoxElem> = self
            .candles
            .iter()
            .enumerate()
            .map(|(i, c)| {
                let color = if c.close >= c.open { GREEN } else { RED };
                BoxElem::new(
                    i as f64,
                    BoxSpread::new(c.low, c.open.min(c.close), (c.open + c.close) / 2.0, c.open.max(c.close), c.high),
                )
                .whisker_width(0.0)
                .box_width(0.7)
                .fill(color)
                .stroke(Stroke::new(1.0, color))
            })
            .collect();

        let close_line: PlotPoints = self
            .candles
            .iter()
            .enumerate()
            .map(|(i, c)| [i as f64, c.close])
            .collect();

        Plot::new("chart")
            .legend(Legend::default())
            .height(500.0)
            .show(ui, |plot_ui| {
                plot_ui.box_plot(BoxPlot::new(boxes).name("OHLC"));
                plot_ui.line(Line::new(close_line).color(ACCENT).name("close"));
                if let Some(sig) = &self.signal {
                    if sig.entry_price > 0.0 {
                        plot_ui.hline(HLine::new(sig.entry_price).color(Color32::LIGHT_BLUE).name("entry"));
                    }
                    if sig.stop_loss > 0.0 {
                        plot_ui.hline(HLine::new(sig.stop_loss).color(RED).name("stop"));
                    }
                    if sig.take_profit > 0.0 {
                        plot_ui.hline(HLine::new(sig.take_profit).color(GREEN).name("target"));
                    }
                }
            });
    }

    fn ui_signal(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label("Symbol");
            ui.text_edit_singleline(&mut self.symbol);
            if ui.button("Generate signal").clicked() {
                self.request_signal();
            }
        });
        ui.separator();
        let Some(sig) = self.signal.clone() else {
            ui.label(RichText::new("No signal yet.").weak());
            return;
        };
        let color = match sig.direction.as_str() {
            "LONG" => GREEN,
            "SHORT" => RED,
            _ => Color32::LIGHT_GRAY,
        };
        ui.horizontal(|ui| {
            ui.label(RichText::new(&sig.symbol).strong().size(22.0));
            ui.label(RichText::new(&sig.direction).color(color).strong().size(22.0));
            ui.label(format!("{:.1}% conf", sig.confidence * 100.0));
        });
        ui.label(format!(
            "entry {:.4}   stop {:.4}   target {:.4}   R:R {:.2}",
            sig.entry_price, sig.stop_loss, sig.take_profit, sig.risk_reward
        ));
        ui.label(format!(
            "regime {}  divergence {}  vol {}  trend {:+.2}  confluence {}",
            sig.regime, sig.divergence, sig.vol_state, sig.trend_score, sig.confluence
        ));
        ui.separator();
        egui::ScrollArea::vertical().show(ui, |ui| {
            for phase in &sig.phases {
                ui.group(|ui| {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new(format!("Phase {}", phase.phase)).weak());
                        ui.label(RichText::new(phase.verdict.as_str()).color(verdict_color(phase.verdict.as_str())).strong());
                        ui.label(RichText::new(&phase.title).strong());
                    });
                    ui.label(&phase.detail);
                });
            }
            if !sig.reasoning.is_empty() {
                ui.group(|ui| {
                    ui.label(RichText::new("Reasoning").strong());
                    ui.label(&sig.reasoning);
                });
            }
        });
    }

    fn ui_chat(&mut self, ui: &mut egui::Ui) {
        egui::ScrollArea::vertical()
            .stick_to_bottom(true)
            .max_height(ui.available_height() - 60.0)
            .show(ui, |ui| {
                for (u, a) in &self.chat_log {
                    ui.group(|ui| {
                        ui.label(RichText::new("you").color(ACCENT).strong());
                        ui.label(u);
                    });
                    if !a.is_empty() {
                        ui.group(|ui| {
                            ui.label(RichText::new("paladin").color(GREEN).strong());
                            ui.label(a);
                        });
                    }
                }
            });
        ui.horizontal(|ui| {
            let resp = ui.add(egui::TextEdit::singleline(&mut self.chat_input).desired_width(f32::INFINITY).hint_text("ask paladin..."));
            if (resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter))) || ui.button("Send").clicked() {
                self.send_chat();
            }
        });
    }

    fn ui_settings(&mut self, ui: &mut egui::Ui) {
        ui.label(RichText::new("API").strong());
        ui.horizontal(|ui| {
            ui.label("Base URL");
            ui.text_edit_singleline(&mut self.config.api_base_url);
        });
        ui.horizontal(|ui| {
            ui.label("Model");
            ui.text_edit_singleline(&mut self.config.model);
        });
        let mut key = self.config.api_key.clone().unwrap_or_default();
        ui.horizontal(|ui| {
            ui.label("API Key");
            if ui.add(egui::TextEdit::singleline(&mut key).password(true)).changed() {
                self.config.api_key = if key.is_empty() { None } else { Some(key.clone()) };
            }
        });
        ui.separator();
        ui.label(RichText::new("Defaults").strong());
        ui.horizontal(|ui| {
            ui.label("Symbol");
            ui.text_edit_singleline(&mut self.config.default_symbol);
        });
        ui.horizontal(|ui| {
            ui.label("Interval");
            ui.text_edit_singleline(&mut self.config.default_interval);
        });
        ui.separator();
        if ui.button("Save").clicked() {
            match self.config.save() {
                Ok(()) => self.status = "config saved".into(),
                Err(e) => self.status = format!("save failed: {e}"),
            }
        }
    }
}

fn verdict_color(v: &str) -> Color32 {
    match v {
        "BULLISH" => GREEN,
        "BEARISH" => RED,
        "CAUTION" => Color32::from_rgb(0xe6, 0xb3, 0x3a),
        _ => Color32::LIGHT_GRAY,
    }
}

impl eframe::App for PaladinApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.drain_events(ctx);
        egui::TopBottomPanel::top("top").show(ctx, |ui| {
            ui.add_space(6.0);
            self.ui_top(ui);
            ui.add_space(4.0);
        });
        egui::CentralPanel::default().show(ctx, |ui| match self.tab {
            Tab::Chart => self.ui_chart(ui),
            Tab::Signal => self.ui_signal(ui),
            Tab::Chat => self.ui_chat(ui),
            Tab::Settings => self.ui_settings(ui),
        });
    }
}
