use netwatch_core::{Config, TimeRange};
use netwatch_db::{
    DaemonStatus, GraphPoint, HistoryEntry, InterfaceDetail, InterfaceStats, Totals,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Page {
    Home,
    Interfaces,
    InterfaceDetail,
    History,
    Graph,
    Live,
    Search,
}

pub struct App {
    pub config: Config,
    pub page: Page,
    pub previous_page: Page,
    pub should_quit: bool,
    pub show_help: bool,
    pub selection: usize,
    pub history_range_idx: usize,
    pub graph_resolution_idx: usize,
    pub search_query: String,

    pub totals: Totals,
    pub speeds: Totals,
    pub interfaces: Vec<InterfaceStats>,
    pub interface_detail: Option<InterfaceDetail>,
    pub selected_interface_id: Option<i64>,
    pub history: Vec<HistoryEntry>,
    pub graph_points: Vec<GraphPoint>,
    pub daemon_status: DaemonStatus,
    pub db_size: u64,
    pub alert_count: usize,
    pub filtered_interfaces: Vec<usize>,
}

impl App {
    pub fn new(config: Config, initial_page: Page) -> Self {
        Self {
            config,
            page: initial_page,
            previous_page: Page::Home,
            should_quit: false,
            show_help: false,
            selection: 0,
            history_range_idx: 0,
            graph_resolution_idx: 0,
            search_query: String::new(),
            totals: Totals {
                download: 0,
                upload: 0,
                rx_rate: 0,
                tx_rate: 0,
            },
            speeds: Totals {
                download: 0,
                upload: 0,
                rx_rate: 0,
                tx_rate: 0,
            },
            interfaces: Vec::new(),
            interface_detail: None,
            selected_interface_id: None,
            history: Vec::new(),
            graph_points: Vec::new(),
            daemon_status: DaemonStatus {
                running: false,
                pid: None,
                last_heartbeat: None,
                sample_interval: None,
            },
            db_size: 0,
            alert_count: 0,
            filtered_interfaces: Vec::new(),
        }
    }

    pub async fn refresh(&mut self, db: &netwatch_db::Database) -> anyhow::Result<()> {
        self.totals = db.today_totals().await?;
        self.speeds = db.current_speeds().await?;
        self.interfaces = db.interface_stats_today().await?;
        self.daemon_status = db.daemon_status().await?;
        self.db_size = db.database_size_bytes().await?;
        self.alert_count = db.unacknowledged_alerts().await?.len();
        self.filtered_interfaces = (0..self.interfaces.len()).collect();

        if let Some(id) = self.selected_interface_id {
            self.interface_detail = Some(db.interface_detail(id).await?);
        }

        let range = self.current_history_range();
        let now = chrono::Utc::now();
        let (start, end) = range.bounds(now);
        self.history = db.history_table(start.timestamp(), end.timestamp()).await?;

        let graph_range = self.graph_range();
        let (gstart, gend) = graph_range.bounds(now);
        self.graph_points = db
            .graph_series(
                gstart.timestamp(),
                gend.timestamp(),
                self.selected_interface_id,
            )
            .await?;

        Ok(())
    }

    pub fn handle_enter(&mut self) {
        match self.page {
            Page::Interfaces => {
                if let Some(idx) = self.filtered_interfaces.get(self.selection) {
                    if let Some(iface) = self.interfaces.get(*idx) {
                        self.selected_interface_id = Some(iface.id);
                        self.previous_page = Page::Interfaces;
                        self.page = Page::InterfaceDetail;
                    }
                }
            }
            Page::History => {}
            _ => {}
        }
    }

    pub fn move_selection(&mut self, delta: i32) {
        let len = match self.page {
            Page::Interfaces => self.filtered_interfaces.len(),
            Page::History => self.history.len(),
            _ => 0,
        };
        if len == 0 {
            return;
        }
        let next = self.selection as i32 + delta;
        self.selection = next.clamp(0, len as i32 - 1) as usize;
    }

    pub fn adjust_range(&mut self, delta: i32) {
        match self.page {
            Page::History => {
                let next = self.history_range_idx as i32 + delta;
                self.history_range_idx = next.clamp(0, 6) as usize;
            }
            Page::Graph => {
                let next = self.graph_resolution_idx as i32 + delta;
                self.graph_resolution_idx = next.clamp(0, 3) as usize;
            }
            _ => {}
        }
    }

    pub fn next_history_range(&mut self) {
        self.history_range_idx = (self.history_range_idx + 1) % 7;
    }

    pub fn current_history_range(&self) -> TimeRange {
        match self.history_range_idx {
            0 => TimeRange::Today,
            1 => TimeRange::Yesterday,
            2 => TimeRange::Last7Days,
            3 => TimeRange::Last30Days,
            4 => TimeRange::CurrentMonth,
            5 => TimeRange::PreviousMonth,
            _ => TimeRange::ThisYear,
        }
    }

    pub fn graph_range(&self) -> TimeRange {
        match self.graph_resolution_idx {
            0 => TimeRange::Today,
            1 => TimeRange::Last7Days,
            2 => TimeRange::Last30Days,
            _ => TimeRange::ThisYear,
        }
    }

    pub fn history_range_label(&self) -> &'static str {
        match self.history_range_idx {
            0 => "Today",
            1 => "Yesterday",
            2 => "Last 7 Days",
            3 => "Last 30 Days",
            4 => "Current Month",
            5 => "Previous Month",
            _ => "This Year",
        }
    }

    pub fn graph_resolution_label(&self) -> &'static str {
        match self.graph_resolution_idx {
            0 => "Hour",
            1 => "Day",
            2 => "Week",
            _ => "Month",
        }
    }

    pub fn apply_search(&mut self) {
        let q = self.search_query.to_lowercase();
        if q.is_empty() {
            self.filtered_interfaces = (0..self.interfaces.len()).collect();
            return;
        }
        self.filtered_interfaces = self
            .interfaces
            .iter()
            .enumerate()
            .filter(|(_, iface)| iface.name.to_lowercase().contains(&q))
            .map(|(i, _)| i)
            .collect();
        self.selection = 0;
    }
}
