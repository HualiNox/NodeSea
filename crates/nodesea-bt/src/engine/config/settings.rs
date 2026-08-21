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

impl From<AlertCategory> for i64 {
    fn from(categories: AlertCategory) -> Self {
        i64::from(categories.bits())
    }
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
   DeprecatedGuidedReadCache => "deprecated_guided_read_cache" = 26,
   NoAtimeStorage => "no_atime_storage" = 27,
   IncomingStartsQueuedTorrents => "incoming_starts_queued_torrents" = 28,
   ReportTrueDownloaded => "report_true_downloaded" = 29,
   StrictEndGameMode => "strict_end_game_mode" = 30,
   DeprecatedBroadcastLsd => "deprecated_broadcast_lsd" = 31,
   EnableOutgoingUtp => "enable_outgoing_utp" = 32,
   EnableIncomingUtp => "enable_incoming_utp" = 33,
   EnableOutgoingTcp => "enable_outgoing_tcp" = 34,
   EnableIncomingTcp => "enable_incoming_tcp" = 35,
   DeprecatedIgnoreResumeTimestamps => "deprecated_ignore_resume_timestamps" = 36,
   NoRecheckIncompleteResume => "no_recheck_incomplete_resume" = 37,
   AnonymousMode => "anonymous_mode" = 38,
   ReportWebSeedDownloads => "report_web_seed_downloads" = 39,
   DeprecatedRateLimitUtp => "deprecated_rate_limit_utp" = 40,
   DeprecatedAnnounceDoubleNat => "deprecated_announce_double_nat" = 41,
   SeedingOutgoingConnections => "seeding_outgoing_connections" = 42,
   NoConnectPrivilegedPorts => "no_connect_privileged_ports" = 43,
   SmoothConnects => "smooth_connects" = 44,
   AlwaysSendUserAgent => "always_send_user_agent" = 45,
   ApplyIpFilterToTrackers => "apply_ip_filter_to_trackers" = 46,
   DeprecatedUseDiskReadAhead => "deprecated_use_disk_read_ahead" = 47,
   DeprecatedLockFiles => "deprecated_lock_files" = 48,
   DeprecatedContiguousRecvBuffer => "deprecated_contiguous_recv_buffer" = 49,
   BanWebSeeds => "ban_web_seeds" = 50,
   DeprecatedForceProxy => "deprecated_force_proxy" = 52,
   SupportShareMode => "support_share_mode" = 53,
   ReportRedundantBytes => "report_redundant_bytes" = 55,
   ListenSystemPortFallback => "listen_system_port_fallback" = 56,
   DeprecatedUseDiskCachePool => "deprecated_use_disk_cache_pool" = 57,
   AnnounceCryptoSupport => "announce_crypto_support" = 58,
   EnableUpnp => "enable_upnp" = 59,
   EnableNatpmp => "enable_natpmp" = 60,
   EnableLsd => "enable_lsd" = 61,
   EnableDht => "enable_dht" = 62,
   PreferRc4 => "prefer_rc4" = 63,
   ProxyHostnames => "proxy_hostnames" = 64,
   ProxyPeerConnections => "proxy_peer_connections" = 65,
   AutoSequential => "auto_sequential" = 66,
   ProxyTrackerConnections => "proxy_tracker_connections" = 67,
   EnableIpNotifier => "enable_ip_notifier" = 68,
   DhtPreferVerifiedNodeIds => "dht_prefer_verified_node_ids" = 69,
   DhtRestrictRoutingIps => "dht_restrict_routing_ips" = 70,
   DhtRestrictSearchIps => "dht_restrict_search_ips" = 71,
   DhtExtendedRoutingTable => "dht_extended_routing_table" = 72,
   DhtAggressiveLookups => "dht_aggressive_lookups" = 73,
   DhtPrivacyLookups => "dht_privacy_lookups" = 74,
   DhtEnforceNodeId => "dht_enforce_node_id" = 75,
   DhtIgnoreDarkInternet => "dht_ignore_dark_internet" = 76,
   DhtReadOnly => "dht_read_only" = 77,
   PieceExtentAffinity => "piece_extent_affinity" = 78,
   ValidateHttpsTrackers => "validate_https_trackers" = 79,
   SsrfMitigation => "ssrf_mitigation" = 80,
   AllowIdna => "allow_idna" = 81,
   EnableSetFileValidData => "enable_set_file_valid_data" = 82,
   Socks5UdpSendLocalEp => "socks5_udp_send_local_ep" = 83,
   ProxySendHostInConnect => "proxy_send_host_in_connect" = 84,
   DiskDisableCopyOnWrite => "disk_disable_copy_on_write" = 85,
   AllowMultipleConnectionsPerPid => "allow_multiple_connections_per_pid" = 86,
   ApplyFilterToDht => "apply_filter_to_dht" = 87,
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
   ActiveDownloads => "active_downloads" = 41,
   ActiveSeeds => "active_seeds" = 42,
   ActiveChecking => "active_checking" = 43,
   ActiveDhtLimit => "active_dht_limit" = 44,
   ActiveTrackerLimit => "active_tracker_limit" = 45,
   ActiveLsdLimit => "active_lsd_limit" = 46,
   ActiveLimit => "active_limit" = 47,
   DeprecatedActiveLoadedLimit => "deprecated_active_loaded_limit" = 48,
   AutoManageInterval => "auto_manage_interval" = 49,
   SeedTimeLimit => "seed_time_limit" = 50,
   AutoScrapeInterval => "auto_scrape_interval" = 51,
   AutoScrapeMinInterval => "auto_scrape_min_interval" = 52,
   MaxPeerlistSize => "max_peerlist_size" = 53,
   MaxPausedPeerlistSize => "max_paused_peerlist_size" = 54,
   MinAnnounceInterval => "min_announce_interval" = 55,
   AutoManageStartup => "auto_manage_startup" = 56,
   SeedingPieceQuota => "seeding_piece_quota" = 57,
   MaxRejects => "max_rejects" = 58,
   RecvSocketBufferSize => "recv_socket_buffer_size" = 59,
   SendSocketBufferSize => "send_socket_buffer_size" = 60,
   MaxPeerRecvBufferSize => "max_peer_recv_buffer_size" = 61,
   DeprecatedFileChecksDelayPerBlock => "deprecated_file_checks_delay_per_block" = 62,
   ReadCacheLineSize => "read_cache_line_size" = 63,
   WriteCacheLineSize => "write_cache_line_size" = 64,
   OptimisticDiskRetry => "optimistic_disk_retry" = 65,
   MaxSuggestPieces => "max_suggest_pieces" = 66,
   LocalServiceAnnounceInterval => "local_service_announce_interval" = 67,
   DhtAnnounceInterval => "dht_announce_interval" = 68,
   UdpTrackerTokenExpiry => "udp_tracker_token_expiry" = 69,
   DeprecatedDefaultCacheMinAge => "deprecated_default_cache_min_age" = 70,
   NumOptimisticUnchokeSlots => "num_optimistic_unchoke_slots" = 71,
   DeprecatedDefaultEstReciprocationRate => "deprecated_default_est_reciprocation_rate" = 72,
   DeprecatedIncreaseEstReciprocationRate => "deprecated_increase_est_reciprocation_rate" = 73,
   DeprecatedDecreaseEstReciprocationRate => "deprecated_decrease_est_reciprocation_rate" = 74,
   MaxPexPeers => "max_pex_peers" = 75,
   TickInterval => "tick_interval" = 76,
   ShareModeTarget => "share_mode_target" = 77,
   UploadRateLimit => "upload_rate_limit" = 78,
   DownloadRateLimit => "download_rate_limit" = 79,
   DeprecatedLocalUploadRateLimit => "deprecated_local_upload_rate_limit" = 80,
   DeprecatedLocalDownloadRateLimit => "deprecated_local_download_rate_limit" = 81,
   DhtUploadRateLimit => "dht_upload_rate_limit" = 82,
   UnchokeSlotsLimit => "unchoke_slots_limit" = 83,
   DeprecatedHalfOpenLimit => "deprecated_half_open_limit" = 84,
   ConnectionsLimit => "connections_limit" = 85,
   ConnectionsSlack => "connections_slack" = 86,
   UtpTargetDelay => "utp_target_delay" = 87,
   UtpGainFactor => "utp_gain_factor" = 88,
   UtpMinTimeout => "utp_min_timeout" = 89,
   UtpSynResends => "utp_syn_resends" = 90,
   UtpFinResends => "utp_fin_resends" = 91,
   UtpNumResends => "utp_num_resends" = 92,
   UtpConnectTimeout => "utp_connect_timeout" = 93,
   DeprecatedUtpDelayedAck => "deprecated_utp_delayed_ack" = 94,
   UtpLossMultiplier => "utp_loss_multiplier" = 95,
   MixedModeAlgorithm => "mixed_mode_algorithm" = 96,
   ListenQueueSize => "listen_queue_size" = 97,
   TorrentConnectBoost => "torrent_connect_boost" = 98,
   AlertQueueSize => "alert_queue_size" = 99,
   MaxMetadataSize => "max_metadata_size" = 100,
   HashingThreads => "hashing_threads" = 101,
   CheckingMemUsage => "checking_mem_usage" = 102,
   PredictivePieceAnnounce => "predictive_piece_announce" = 103,
   AioThreads => "aio_threads" = 104,
   DeprecatedAioMax => "deprecated_aio_max" = 105,
   DeprecatedNetworkThreads => "deprecated_network_threads" = 106,
   DeprecatedSslListen => "deprecated_ssl_listen" = 107,
   TrackerBackoff => "tracker_backoff" = 108,
   ShareRatioLimit => "share_ratio_limit" = 109,
   SeedTimeRatioLimit => "seed_time_ratio_limit" = 110,
   PeerTurnover => "peer_turnover" = 111,
   PeerTurnoverCutoff => "peer_turnover_cutoff" = 112,
   PeerTurnoverInterval => "peer_turnover_interval" = 113,
   ConnectSeedEveryNDownload => "connect_seed_every_n_download" = 114,
   MaxHttpRecvBufferSize => "max_http_recv_buffer_size" = 115,
   MaxRetryPortBind => "max_retry_port_bind" = 116,
   AlertMask => "alert_mask" = 117,
   OutEncPolicy => "out_enc_policy" = 118,
   InEncPolicy => "in_enc_policy" = 119,
   AllowedEncLevel => "allowed_enc_level" = 120,
   InactiveDownRate => "inactive_down_rate" = 121,
   InactiveUpRate => "inactive_up_rate" = 122,
   ProxyType => "proxy_type" = 123,
   ProxyPort => "proxy_port" = 124,
   I2pPort => "i2p_port" = 125,
   DeprecatedCacheSizeVolatile => "deprecated_cache_size_volatile" = 126,
   UrlseedMaxRequestBytes => "urlseed_max_request_bytes" = 127,
   WebSeedNameLookupRetry => "web_seed_name_lookup_retry" = 128,
   CloseFileInterval => "close_file_interval" = 129,
   UtpCwndReduceTimer => "utp_cwnd_reduce_timer" = 130,
   MaxWebSeedConnections => "max_web_seed_connections" = 131,
   ResolverCacheTimeout => "resolver_cache_timeout" = 132,
   SendNotSentLowWatermark => "send_not_sent_low_watermark" = 133,
   RateChokerInitialThreshold => "rate_choker_initial_threshold" = 134,
   UpnpLeaseDuration => "upnp_lease_duration" = 135,
   MaxConcurrentHttpAnnounces => "max_concurrent_http_announces" = 136,
   DhtMaxPeersReply => "dht_max_peers_reply" = 137,
   DhtSearchBranching => "dht_search_branching" = 138,
   DhtMaxFailCount => "dht_max_fail_count" = 139,
   DhtMaxTorrents => "dht_max_torrents" = 140,
   DhtMaxDhtItems => "dht_max_dht_items" = 141,
   DhtMaxPeers => "dht_max_peers" = 142,
   DhtMaxTorrentSearchReply => "dht_max_torrent_search_reply" = 143,
   DhtBlockTimeout => "dht_block_timeout" = 144,
   DhtBlockRatelimit => "dht_block_ratelimit" = 145,
   DhtItemLifetime => "dht_item_lifetime" = 146,
   DhtSampleInfohashesInterval => "dht_sample_infohashes_interval" = 147,
   DhtMaxInfohashesSampleCount => "dht_max_infohashes_sample_count" = 148,
   MaxPieceCount => "max_piece_count" = 149,
   MetadataTokenLimit => "metadata_token_limit" = 150,
   DiskWriteMode => "disk_write_mode" = 151,
   MmapFileSizeCutoff => "mmap_file_size_cutoff" = 152,
   I2pInboundQuantity => "i2p_inbound_quantity" = 153,
   I2pOutboundQuantity => "i2p_outbound_quantity" = 154,
   I2pInboundLength => "i2p_inbound_length" = 155,
   I2pOutboundLength => "i2p_outbound_length" = 156,
   AnnouncePort => "announce_port" = 157,
   I2pInboundLengthVariance => "i2p_inbound_length_variance" = 158,
   I2pOutboundLengthVariance => "i2p_outbound_length_variance" = 159,
   NatpmpLeaseDuration => "natpmp_lease_duration" = 160,
   MinWebsocketAnnounceInterval => "min_websocket_announce_interval" = 161,
   WebtorrentConnectionTimeout => "webtorrent_connection_timeout" = 162,
   MaxWebtorrentOffers => "max_webtorrent_offers" = 163,
);

impl IntSetting {
    /// Deprecated alias for [`IntSetting::PeerDscp`].
    #[allow(non_upper_case_globals)]
    pub const PeerTos: Self = Self::PeerDscp;
}

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
    pub fn set_int(&mut self, key: IntSetting, value: impl Into<i64>) {
        let value = value.into();
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

    /// Consumes the pack and returns its pending setting updates.
    pub(crate) fn into_values(self) -> Vec<Setting> {
        self.values
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn alert_categories_can_be_set_directly() {
        let categories = AlertCategory::DHT_NOTIFICATION | AlertCategory::ERROR_NOTIFICATION;
        let mut settings = SettingsPack::new();

        settings.set_int(IntSetting::AlertMask, categories);

        assert_eq!(
            settings.values(),
            &[Setting::Int(
                IntSetting::AlertMask,
                i64::from(categories.bits())
            )]
        );
    }
}
