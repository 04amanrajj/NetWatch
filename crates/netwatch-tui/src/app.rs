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

}}}}
