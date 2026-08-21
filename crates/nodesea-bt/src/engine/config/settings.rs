//! Rust representation of the libtorrent settings_pack keys.

use super::macros::define_settings;
use bitflags::bitflags;

const STRING_TYPE_BASE: u16 = 0x0000;
const INT_TYPE_BASE: u16 = 0x4000;
const BOOL_TYPE_BASE: u16 = 0x8000;

bitflags! {
    /// Bit flags representing libtorrent alert categories.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct AlertCategory: u32 {
        /// Error notifications.
        const ERROR_NOTIFICATION = 1 << 0;
        /// Peer notifications.
        const PEER_NOTIFICATION = 1 << 1;
        /// Port mapping notifications.
        const PORT_MAPPING_NOTIFICATION = 1 << 2;
        /// Storage notifications.
        const STORAGE_NOTIFICATION = 1 << 3;
        /// Tracker notifications.
        const TRACKER_NOTIFICATION = 1 << 4;
        /// Connection notifications.
        const CONNECT_NOTIFICATION = 1 << 5;
        /// Status notifications.
        const STATUS_NOTIFICATION = 1 << 6;
        /// IP block notifications.
        const IP_BLOCK_NOTIFICATION = 1 << 8;
        /// Performance warnings.
        const PERFORMANCE_WARNING = 1 << 9;
        /// DHT notifications.
        const DHT_NOTIFICATION = 1 << 10;
        /// Session log notifications.
        const SESSION_LOG_NOTIFICATION = 1 << 13;
        /// Torrent log notifications.
        const TORRENT_LOG_NOTIFICATION = 1 << 14;
        /// Peer log notifications.
        const PEER_LOG_NOTIFICATION = 1 << 15;
        /// Incoming request notifications.
        const INCOMING_REQUEST_NOTIFICATION = 1 << 16;
        /// DHT log notifications.
        const DHT_LOG_NOTIFICATION = 1 << 17;
        /// DHT operation notifications.
        const DHT_OPERATION_NOTIFICATION = 1 << 18;
        /// Port mapping log notifications.
        const PORT_MAPPING_LOG_NOTIFICATION = 1 << 19;
        /// Piece picker log notifications.
        const PICKER_LOG_NOTIFICATION = 1 << 20;
        /// File progress notifications.
        const FILE_PROGRESS_NOTIFICATION = 1 << 21;
        /// Piece progress notifications.
        const PIECE_PROGRESS_NOTIFICATION = 1 << 22;
        /// Upload notifications.
        const UPLOAD_NOTIFICATION = 1 << 23;
        /// Block progress notifications.
        const BLOCK_PROGRESS_NOTIFICATION = 1 << 24;
    }
}

impl AlertCategory {
    /// Enables all alert category bits.
    pub const ALL: Self = Self::from_bits_retain(u32::MAX);
}

define_settings!(StringSetting, "String settings defined by the current libtorrent ABI.", STRING_TYPE_BASE;
   UserAgent => "user_agent" = 0,
   AnnounceIp => "announce_ip" = 1,
   DeprecatedMmapCache => "deprecated_mmap_cache" = 2,
   HandshakeClientVersion => "handshake_client_version" = 3,
   OutgoingInterfaces => "outgoing_interfaces" = 4,
   ListenInterfaces => "listen_interfaces" = 5,
   ProxyHostname => "proxy_hostname" = 6,
   ProxyUsername => "proxy_username" = 7,
   ProxyPassword => "proxy_password" = 8,
   I2pHostname => "i2p_hostname" = 9,
   PeerFingerprint => "peer_fingerprint" = 10,
   DhtBootstrapNodes => "dht_bootstrap_nodes" = 11,
   NatpmpGateway => "natpmp_gateway" = 12,
   WebtorrentStunServer => "webtorrent_stun_server" = 13,
);

define_settings!(BoolSetting, "Boolean settings defined by the current libtorrent ABI.", BOOL_TYPE_BASE;
   AllowMultipleConnectionsPerIp => "allow_multiple_connections_per_ip" = 0,
   DeprecatedIgnoreLimitsOnLocalNetwork => "deprecated_ignore_limits_on_local_network" = 1,
   SendRedundantHave => "send_redundant_have" = 2,
   DeprecatedLazyBitfield => "deprecated_lazy_bitfield" = 3,
   UseDhtAsFallback => "use_dht_as_fallback" = 4,
   UpnpIgnoreNonrouters => "upnp_ignore_nonrouters" = 5,
   UseParoleMode => "use_parole_mode" = 6,
   DeprecatedUseReadCache => "deprecated_use_read_cache" = 7,
   DeprecatedUseWriteCache => "deprecated_use_write_cache" = 8,
   DeprecatedFlushWriteCache => "deprecated_flush_write_cache" = 9,
   DeprecatedCoalesceReads => "deprecated_coalesce_reads" = 10,
   DeprecatedCoalesceWrites => "deprecated_coalesce_writes" = 11,
   AutoManagePreferSeeds => "auto_manage_prefer_seeds" = 12,
   DontCountSlowTorrents => "dont_count_slow_torrents" = 13,
   CloseRedundantConnections => "close_redundant_connections" = 14,
   PrioritizePartialPieces => "prioritize_partial_pieces" = 15,
   RateLimitIpOverhead => "rate_limit_ip_overhead" = 16,
   AnnounceToAllTiers => "announce_to_all_tiers" = 17,
   AnnounceToAllTrackers => "announce_to_all_trackers" = 18,
   PreferUdpTrackers => "prefer_udp_trackers" = 19,
   DeprecatedStrictSuperSeeding => "deprecated_strict_super_seeding" = 20,
   DeprecatedLockDiskCache => "deprecated_lock_disk_cache" = 21,
   DisableHashChecks => "disable_hash_checks" = 22,
   AllowI2pMixed => "allow_i2p_mixed" = 23,
   DeprecatedLowPrioDisk => "deprecated_low_prio_disk" = 24,
   DeprecatedGuidedReadCache => "deprecated_guided_read_cache" = 25,
   NoAtimeStorage => "no_atime_storage" = 26,
   IncomingStartsQueuedTorrents => "incoming_starts_queued_torrents" = 27,
   ReportTrueDownloaded => "report_true_downloaded" = 28,
   StrictEndGameMode => "strict_end_game_mode" = 29,
   DeprecatedBroadcastLsd => "deprecated_broadcast_lsd" = 30,
   EnableOutgoingUtp => "enable_outgoing_utp" = 31,
   EnableIncomingUtp => "enable_incoming_utp" = 32,
   EnableOutgoingTcp => "enable_outgoing_tcp" = 33,
   EnableIncomingTcp => "enable_incoming_tcp" = 34,
   DeprecatedIgnoreResumeTimestamps => "deprecated_ignore_resume_timestamps" = 35,
   NoRecheckIncompleteResume => "no_recheck_incomplete_resume" = 36,
   AnonymousMode => "anonymous_mode" = 37,
   ReportWebSeedDownloads => "report_web_seed_downloads" = 38,
   DeprecatedRateLimitUtp => "deprecated_rate_limit_utp" = 39,
   DeprecatedAnnounceDoubleNat => "deprecated_announce_double_nat" = 40,
   SeedingOutgoingConnections => "seeding_outgoing_connections" = 41,
   NoConnectPrivilegedPorts => "no_connect_privileged_ports" = 42,
   SmoothConnects => "smooth_connects" = 43,
   AlwaysSendUserAgent => "always_send_user_agent" = 44,
   ApplyIpFilterToTrackers => "apply_ip_filter_to_trackers" = 45,
   DeprecatedUseDiskReadAhead => "deprecated_use_disk_read_ahead" = 46,
   DeprecatedLockFiles => "deprecated_lock_files" = 47,
   DeprecatedContiguousRecvBuffer => "deprecated_contiguous_recv_buffer" = 48,
   BanWebSeeds => "ban_web_seeds" = 49,
   DeprecatedForceProxy => "deprecated_force_proxy" = 50,
   SupportShareMode => "support_share_mode" = 51,
   ReportRedundantBytes => "report_redundant_bytes" = 52,
   ListenSystemPortFallback => "listen_system_port_fallback" = 53,
   DeprecatedUseDiskCachePool => "deprecated_use_disk_cache_pool" = 54,
   AnnounceCryptoSupport => "announce_crypto_support" = 55,
   EnableUpnp => "enable_upnp" = 56,
   EnableNatpmp => "enable_natpmp" = 57,
   EnableLsd => "enable_lsd" = 58,
   EnableDht => "enable_dht" = 59,
   PreferRc4 => "prefer_rc4" = 60,
   ProxyHostnames => "proxy_hostnames" = 61,
   ProxyPeerConnections => "proxy_peer_connections" = 62,
   AutoSequential => "auto_sequential" = 63,
   ProxyTrackerConnections => "proxy_tracker_connections" = 64,
   EnableIpNotifier => "enable_ip_notifier" = 65,
   DhtPreferVerifiedNodeIds => "dht_prefer_verified_node_ids" = 66,
   DhtRestrictRoutingIps => "dht_restrict_routing_ips" = 67,
   DhtRestrictSearchIps => "dht_restrict_search_ips" = 68,
   DhtExtendedRoutingTable => "dht_extended_routing_table" = 69,
   DhtAggressiveLookups => "dht_aggressive_lookups" = 70,
   DhtPrivacyLookups => "dht_privacy_lookups" = 71,
   DhtEnforceNodeId => "dht_enforce_node_id" = 72,
   DhtIgnoreDarkInternet => "dht_ignore_dark_internet" = 73,
   DhtReadOnly => "dht_read_only" = 74,
   PieceExtentAffinity => "piece_extent_affinity" = 75,
   ValidateHttpsTrackers => "validate_https_trackers" = 76,
   SsrfMitigation => "ssrf_mitigation" = 77,
   AllowIdna => "allow_idna" = 78,
   EnableSetFileValidData => "enable_set_file_valid_data" = 79,
   Socks5UdpSendLocalEp => "socks5_udp_send_local_ep" = 80,
   ProxySendHostInConnect => "proxy_send_host_in_connect" = 81,
   DiskDisableCopyOnWrite => "disk_disable_copy_on_write" = 82,
   AllowMultipleConnectionsPerPid => "allow_multiple_connections_per_pid" = 83,
   ApplyFilterToDht => "apply_filter_to_dht" = 84,
);

define_settings!(IntSetting, "Integer settings defined by the current libtorrent ABI.", INT_TYPE_BASE;
   TrackerCompletionTimeout => "tracker_completion_timeout" = 0,
   TrackerReceiveTimeout => "tracker_receive_timeout" = 1,
   StopTrackerTimeout => "stop_tracker_timeout" = 2,
   TrackerMaximumResponseLength => "tracker_maximum_response_length" = 3,
   PieceTimeout => "piece_timeout" = 4,
   RequestTimeout => "request_timeout" = 5,
   RequestQueueTime => "request_queue_time" = 6,
   MaxAllowedInRequestQueue => "max_allowed_in_request_queue" = 7,
   MaxOutRequestQueue => "max_out_request_queue" = 8,
   WholePiecesThreshold => "whole_pieces_threshold" = 9,
   PeerTimeout => "peer_timeout" = 10,
   UrlseedTimeout => "urlseed_timeout" = 11,
   UrlseedPipelineSize => "urlseed_pipeline_size" = 12,
   UrlseedWaitRetry => "urlseed_wait_retry" = 13,
   FilePoolSize => "file_pool_size" = 14,
   MaxFailcount => "max_failcount" = 15,
   MinReconnectTime => "min_reconnect_time" = 16,
   PeerConnectTimeout => "peer_connect_timeout" = 17,
   ConnectionSpeed => "connection_speed" = 18,
   InactivityTimeout => "inactivity_timeout" = 19,
   UnchokeInterval => "unchoke_interval" = 20,
   OptimisticUnchokeInterval => "optimistic_unchoke_interval" = 21,
   NumWant => "num_want" = 22,
   InitialPickerThreshold => "initial_picker_threshold" = 23,
   AllowedFastSetSize => "allowed_fast_set_size" = 24,
   SuggestMode => "suggest_mode" = 25,
   MaxQueuedDiskBytes => "max_queued_disk_bytes" = 26,
   HandshakeTimeout => "handshake_timeout" = 27,
   SendBufferLowWatermark => "send_buffer_low_watermark" = 28,
   SendBufferWatermark => "send_buffer_watermark" = 29,
   SendBufferWatermarkFactor => "send_buffer_watermark_factor" = 30,
   ChokingAlgorithm => "choking_algorithm" = 31,
   SeedChokingAlgorithm => "seed_choking_algorithm" = 32,
   DeprecatedCacheSize => "deprecated_cache_size" = 33,
   DeprecatedCacheBufferChunkSize => "deprecated_cache_buffer_chunk_size" = 34,
   DeprecatedCacheExpiry => "deprecated_cache_expiry" = 35,
   DiskIoWriteMode => "disk_io_write_mode" = 36,
   DiskIoReadMode => "disk_io_read_mode" = 37,
   OutgoingPort => "outgoing_port" = 38,
   NumOutgoingPorts => "num_outgoing_ports" = 39,
   PeerDscp => "peer_dscp" = 40,
   PeerTos => "peer_tos" = 41,
   ActiveDownloads => "active_downloads" = 42,
   ActiveSeeds => "active_seeds" = 43,
   ActiveChecking => "active_checking" = 44,
   ActiveDhtLimit => "active_dht_limit" = 45,
   ActiveTrackerLimit => "active_tracker_limit" = 46,
   ActiveLsdLimit => "active_lsd_limit" = 47,
   ActiveLimit => "active_limit" = 48,
   DeprecatedActiveLoadedLimit => "deprecated_active_loaded_limit" = 49,
   AutoManageInterval => "auto_manage_interval" = 50,
   SeedTimeLimit => "seed_time_limit" = 51,
   AutoScrapeInterval => "auto_scrape_interval" = 52,
   AutoScrapeMinInterval => "auto_scrape_min_interval" = 53,
   MaxPeerlistSize => "max_peerlist_size" = 54,
   MaxPausedPeerlistSize => "max_paused_peerlist_size" = 55,
   MinAnnounceInterval => "min_announce_interval" = 56,
   AutoManageStartup => "auto_manage_startup" = 57,
   SeedingPieceQuota => "seeding_piece_quota" = 58,
   MaxRejects => "max_rejects" = 59,
   RecvSocketBufferSize => "recv_socket_buffer_size" = 60,
   SendSocketBufferSize => "send_socket_buffer_size" = 61,
   MaxPeerRecvBufferSize => "max_peer_recv_buffer_size" = 62,
   DeprecatedFileChecksDelayPerBlock => "deprecated_file_checks_delay_per_block" = 63,
   ReadCacheLineSize => "read_cache_line_size" = 64,
   WriteCacheLineSize => "write_cache_line_size" = 65,
   OptimisticDiskRetry => "optimistic_disk_retry" = 66,
   MaxSuggestPieces => "max_suggest_pieces" = 67,
   LocalServiceAnnounceInterval => "local_service_announce_interval" = 68,
   DhtAnnounceInterval => "dht_announce_interval" = 69,
   UdpTrackerTokenExpiry => "udp_tracker_token_expiry" = 70,
   DeprecatedDefaultCacheMinAge => "deprecated_default_cache_min_age" = 71,
   NumOptimisticUnchokeSlots => "num_optimistic_unchoke_slots" = 72,
   DeprecatedDefaultEstReciprocationRate => "deprecated_default_est_reciprocation_rate" = 73,
   DeprecatedIncreaseEstReciprocationRate => "deprecated_increase_est_reciprocation_rate" = 74,
   DeprecatedDecreaseEstReciprocationRate => "deprecated_decrease_est_reciprocation_rate" = 75,
   MaxPexPeers => "max_pex_peers" = 76,
   TickInterval => "tick_interval" = 77,
   ShareModeTarget => "share_mode_target" = 78,
   UploadRateLimit => "upload_rate_limit" = 79,
   DownloadRateLimit => "download_rate_limit" = 80,
   DeprecatedLocalUploadRateLimit => "deprecated_local_upload_rate_limit" = 81,
   DeprecatedLocalDownloadRateLimit => "deprecated_local_download_rate_limit" = 82,
   DhtUploadRateLimit => "dht_upload_rate_limit" = 83,
   UnchokeSlotsLimit => "unchoke_slots_limit" = 84,
   DeprecatedHalfOpenLimit => "deprecated_half_open_limit" = 85,
   ConnectionsLimit => "connections_limit" = 86,
   ConnectionsSlack => "connections_slack" = 87,
   UtpTargetDelay => "utp_target_delay" = 88,
   UtpGainFactor => "utp_gain_factor" = 89,
   UtpMinTimeout => "utp_min_timeout" = 90,
   UtpSynResends => "utp_syn_resends" = 91,
   UtpFinResends => "utp_fin_resends" = 92,
   UtpNumResends => "utp_num_resends" = 93,
   UtpConnectTimeout => "utp_connect_timeout" = 94,
   DeprecatedUtpDelayedAck => "deprecated_utp_delayed_ack" = 95,
   UtpLossMultiplier => "utp_loss_multiplier" = 96,
   MixedModeAlgorithm => "mixed_mode_algorithm" = 97,
   ListenQueueSize => "listen_queue_size" = 98,
   TorrentConnectBoost => "torrent_connect_boost" = 99,
   AlertQueueSize => "alert_queue_size" = 100,
   MaxMetadataSize => "max_metadata_size" = 101,
   HashingThreads => "hashing_threads" = 102,
   CheckingMemUsage => "checking_mem_usage" = 103,
   PredictivePieceAnnounce => "predictive_piece_announce" = 104,
   AioThreads => "aio_threads" = 105,
   DeprecatedAioMax => "deprecated_aio_max" = 106,
   DeprecatedNetworkThreads => "deprecated_network_threads" = 107,
   DeprecatedSslListen => "deprecated_ssl_listen" = 108,
   TrackerBackoff => "tracker_backoff" = 109,
   ShareRatioLimit => "share_ratio_limit" = 110,
   SeedTimeRatioLimit => "seed_time_ratio_limit" = 111,
   PeerTurnover => "peer_turnover" = 112,
   PeerTurnoverCutoff => "peer_turnover_cutoff" = 113,
   PeerTurnoverInterval => "peer_turnover_interval" = 114,
   ConnectSeedEveryNDownload => "connect_seed_every_n_download" = 115,
   MaxHttpRecvBufferSize => "max_http_recv_buffer_size" = 116,
   MaxRetryPortBind => "max_retry_port_bind" = 117,
   AlertMask => "alert_mask" = 118,
   OutEncPolicy => "out_enc_policy" = 119,
   InEncPolicy => "in_enc_policy" = 120,
   AllowedEncLevel => "allowed_enc_level" = 121,
   InactiveDownRate => "inactive_down_rate" = 122,
   InactiveUpRate => "inactive_up_rate" = 123,
   ProxyType => "proxy_type" = 124,
   ProxyPort => "proxy_port" = 125,
   I2pPort => "i2p_port" = 126,
   DeprecatedCacheSizeVolatile => "deprecated_cache_size_volatile" = 127,
   UrlseedMaxRequestBytes => "urlseed_max_request_bytes" = 128,
   WebSeedNameLookupRetry => "web_seed_name_lookup_retry" = 129,
   CloseFileInterval => "close_file_interval" = 130,
   UtpCwndReduceTimer => "utp_cwnd_reduce_timer" = 131,
   MaxWebSeedConnections => "max_web_seed_connections" = 132,
   ResolverCacheTimeout => "resolver_cache_timeout" = 133,
   SendNotSentLowWatermark => "send_not_sent_low_watermark" = 134,
   RateChokerInitialThreshold => "rate_choker_initial_threshold" = 135,
   UpnpLeaseDuration => "upnp_lease_duration" = 136,
   MaxConcurrentHttpAnnounces => "max_concurrent_http_announces" = 137,
   DhtMaxPeersReply => "dht_max_peers_reply" = 138,
   DhtSearchBranching => "dht_search_branching" = 139,
   DhtMaxFailCount => "dht_max_fail_count" = 140,
   DhtMaxTorrents => "dht_max_torrents" = 141,
   DhtMaxDhtItems => "dht_max_dht_items" = 142,
   DhtMaxPeers => "dht_max_peers" = 143,
   DhtMaxTorrentSearchReply => "dht_max_torrent_search_reply" = 144,
   DhtBlockTimeout => "dht_block_timeout" = 145,
   DhtBlockRatelimit => "dht_block_ratelimit" = 146,
   DhtItemLifetime => "dht_item_lifetime" = 147,
   DhtSampleInfohashesInterval => "dht_sample_infohashes_interval" = 148,
   DhtMaxInfohashesSampleCount => "dht_max_infohashes_sample_count" = 149,
   MaxPieceCount => "max_piece_count" = 150,
   MetadataTokenLimit => "metadata_token_limit" = 151,
   DiskWriteMode => "disk_write_mode" = 152,
   MmapFileSizeCutoff => "mmap_file_size_cutoff" = 153,
   I2pInboundQuantity => "i2p_inbound_quantity" = 154,
   I2pOutboundQuantity => "i2p_outbound_quantity" = 155,
   I2pInboundLength => "i2p_inbound_length" = 156,
   I2pOutboundLength => "i2p_outbound_length" = 157,
   AnnouncePort => "announce_port" = 158,
   I2pInboundLengthVariance => "i2p_inbound_length_variance" = 159,
   I2pOutboundLengthVariance => "i2p_outbound_length_variance" = 160,
   NatpmpLeaseDuration => "natpmp_lease_duration" = 161,
   MinWebsocketAnnounceInterval => "min_websocket_announce_interval" = 162,
   WebtorrentConnectionTimeout => "webtorrent_connection_timeout" = 163,
   MaxWebtorrentOffers => "max_webtorrent_offers" = 164,
);

/// A typed libtorrent setting update.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Setting {
    /// A string setting.
    String(StringSetting, String),
    /// An integer setting.
    Int(IntSetting, i64),
    /// A boolean setting.
    Bool(BoolSetting, bool),
}

/// A collection of libtorrent settings to apply to a session.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SettingsPack {
    values: Vec<Setting>,
}

impl SettingsPack {
    /// Creates an empty settings pack.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets a string setting.
    pub fn set_string(&mut self, key: StringSetting, value: impl Into<String>) {
        self.values
            .retain(|setting| !matches!(setting, Setting::String(existing, _) if *existing == key));
        self.values.push(Setting::String(key, value.into()));
    }

    /// Sets an integer setting.
    pub fn set_int(&mut self, key: IntSetting, value: i64) {
        self.values
            .retain(|setting| !matches!(setting, Setting::Int(existing, _) if *existing == key));
        self.values.push(Setting::Int(key, value));
    }

    /// Sets a boolean setting.
    pub fn set_bool(&mut self, key: BoolSetting, value: bool) {
        self.values
            .retain(|setting| !matches!(setting, Setting::Bool(existing, _) if *existing == key));
        self.values.push(Setting::Bool(key, value));
    }

    /// Returns the configured setting updates.
    pub fn values(&self) -> &[Setting] {
        &self.values
    }
}
