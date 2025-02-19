use {
    crate::client_error,
    serde::{de, Deserialize, Deserializer, Serialize, Serializer},
    solana_account_decoder_client_types::{token::UiTokenAmount, UiAccount},
    solana_clock::{Epoch, Slot, UnixTimestamp},
    solana_fee_calculator::{FeeCalculator, FeeRateGovernor},
    solana_inflation::Inflation,
    solana_transaction_error::{TransactionError, TransactionResult as Result},
    solana_transaction_status_client_types::{
        ConfirmedTransactionStatusWithSignature, TransactionConfirmationStatus, UiConfirmedBlock,
        UiInnerInstructions, UiTransactionReturnData,
    },
    solana_version::ClientId,
    std::{borrow::Cow, collections::HashMap, fmt, net::SocketAddr, num::{IntErrorKind, ParseIntError}, str::FromStr},
    thiserror::Error,
};

/// Wrapper for rpc return types of methods that provide responses both with and without context.
/// Main purpose of this is to fix methods that lack context information in their return type,
/// without breaking backwards compatibility.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum OptionalContext<T> {
    Context(Response<T>),
    NoContext(T),
}

impl<T> OptionalContext<T> {
    pub fn parse_value(self) -> T {
        match self {
            Self::Context(response) => response.value,
            Self::NoContext(value) => value,
        }
    }
}

pub type RpcResult<T> = client_error::Result<Response<T>>;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RpcResponseContext {
    pub slot: Slot,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_version: Option<RpcApiVersion>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RpcApiVersion(semver::Version);

impl std::ops::Deref for RpcApiVersion {
    type Target = semver::Version;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Default for RpcApiVersion {
    fn default() -> Self {
        Self(solana_version::Version::default().as_semver_version())
    }
}

impl Serialize for RpcApiVersion {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for RpcApiVersion {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s: String = Deserialize::deserialize(deserializer)?;
        Ok(RpcApiVersion(
            semver::Version::from_str(&s).map_err(serde::de::Error::custom)?,
        ))
    }
}

impl RpcResponseContext {
    pub fn new(slot: Slot) -> Self {
        Self {
            slot,
            api_version: Some(RpcApiVersion::default()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Response<T> {
    pub context: RpcResponseContext,
    pub value: T,
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RpcBlockCommitment<T> {
    pub commitment: Option<T>,
    pub total_stake: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RpcBlockhashFeeCalculator {
    pub blockhash: String,
    pub fee_calculator: FeeCalculator,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RpcBlockhash {
    pub blockhash: String,
    pub last_valid_block_height: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RpcFeeCalculator {
    pub fee_calculator: FeeCalculator,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RpcFeeRateGovernor {
    pub fee_rate_governor: FeeRateGovernor,
}

#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RpcInflationGovernor {
    pub initial: f64,
    pub terminal: f64,
    pub taper: f64,
    pub foundation: f64,
    pub foundation_term: f64,
}

impl From<Inflation> for RpcInflationGovernor {
    fn from(inflation: Inflation) -> Self {
        Self {
            initial: inflation.initial,
            terminal: inflation.terminal,
            taper: inflation.taper,
            foundation: inflation.foundation,
            foundation_term: inflation.foundation_term,
        }
    }
}

#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RpcInflationRate {
    pub total: f64,
    pub validator: f64,
    pub foundation: f64,
    pub epoch: Epoch,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RpcKeyedAccount {
    pub pubkey: String,
    pub account: UiAccount,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub struct SlotInfo {
    pub slot: Slot,
    pub parent: Slot,
    pub root: Slot,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SlotTransactionStats {
    pub num_transaction_entries: u64,
    pub num_successful_transactions: u64,
    pub num_failed_transactions: u64,
    pub max_transactions_per_entry: u64,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
#[serde(rename_all = "camelCase", tag = "type")]
pub enum SlotUpdate {
    FirstShredReceived {
        slot: Slot,
        timestamp: u64,
    },
    Completed {
        slot: Slot,
        timestamp: u64,
    },
    CreatedBank {
        slot: Slot,
        parent: Slot,
        timestamp: u64,
    },
    Frozen {
        slot: Slot,
        timestamp: u64,
        stats: SlotTransactionStats,
    },
    Dead {
        slot: Slot,
        timestamp: u64,
        err: String,
    },
    OptimisticConfirmation {
        slot: Slot,
        timestamp: u64,
    },
    Root {
        slot: Slot,
        timestamp: u64,
    },
}

impl SlotUpdate {
    pub fn slot(&self) -> Slot {
        match self {
            Self::FirstShredReceived { slot, .. } => *slot,
            Self::Completed { slot, .. } => *slot,
            Self::CreatedBank { slot, .. } => *slot,
            Self::Frozen { slot, .. } => *slot,
            Self::Dead { slot, .. } => *slot,
            Self::OptimisticConfirmation { slot, .. } => *slot,
            Self::Root { slot, .. } => *slot,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase", untagged)]
pub enum RpcSignatureResult {
    ProcessedSignature(ProcessedSignatureResult),
    ReceivedSignature(ReceivedSignatureResult),
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RpcLogsResponse {
    pub signature: String, // Signature as base58 string
    pub err: Option<TransactionError>,
    pub logs: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ProcessedSignatureResult {
    pub err: Option<TransactionError>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum ReceivedSignatureResult {
    ReceivedSignature,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RpcContactInfo {
    /// Pubkey of the node as a base-58 string
    pub pubkey: String,
    /// Gossip port
    pub gossip: Option<SocketAddr>,
    /// Tvu UDP port
    pub tvu: Option<SocketAddr>,
    /// Tpu UDP port
    pub tpu: Option<SocketAddr>,
    /// Tpu QUIC port
    pub tpu_quic: Option<SocketAddr>,
    /// Tpu UDP forwards port
    pub tpu_forwards: Option<SocketAddr>,
    /// Tpu QUIC forwards port
    pub tpu_forwards_quic: Option<SocketAddr>,
    /// Tpu UDP vote port
    pub tpu_vote: Option<SocketAddr>,
    /// Server repair UDP port
    pub serve_repair: Option<SocketAddr>,
    /// JSON RPC port
    pub rpc: Option<SocketAddr>,
    /// WebSocket PubSub port
    pub pubsub: Option<SocketAddr>,
    /// Software version
    pub version: Option<String>,
    /// First 4 bytes of the FeatureSet identifier
    pub feature_set: Option<u32>,
    /// Shred version
    pub shred_version: Option<u16>,
    /// First 4 bytes of the sha1 commit hash
    #[serde(serialize_with = "to_hex", deserialize_with = "from_hex")]
    pub commit: Option<u32>,
    /// Client id
    pub client_id: Option<ClientId>,
}
// TODO relocate this?
pub fn to_hex<S>(input: &Option<u32>, serializer: S) -> std::result::Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    let sha1_or_0 = input.unwrap_or(0);
    serializer.serialize_str(format!("{:08x}", sha1_or_0).as_str())
}
// https://play.rust-lang.org/?version=stable&mode=debug&edition=2018&gist=ee7f582b5873013723596790a7993925
// https://stackoverflow.com/questions/46753955/how-to-transform-fields-during-deserialization-using-serde


fn from_hex<'de, D>(deserializer: D) -> std::result::Result<Option<u32>, D::Error>
where
    D: Deserializer<'de>,
{
    println!("Running from_hex");
    let input: Cow<'de, str> = serde::Deserialize::deserialize(deserializer)?;
    println!("from_hex input: ->{:?}<-", input);
    let Some(first_8) = &input.get(..8) else {panic!("need a better error here") };
    print!("from_hex first_8: ->{}<-", first_8);
    let foo = u32::from_str_radix(&first_8, /*radix:*/ 16)
        .map_err(|_| serde::de::Error::invalid_type(serde::de::Unexpected::Str(first_8), &"hex u32"))?;
    Ok(Some(foo))
}

/// Map of leader base58 identity pubkeys to the slot indices relative to the first epoch slot
pub type RpcLeaderSchedule = HashMap<String, Vec<usize>>;

#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RpcBlockProductionRange {
    pub first_slot: Slot,
    pub last_slot: Slot,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RpcBlockProduction {
    /// Map of leader base58 identity pubkeys to a tuple of `(number of leader slots, number of blocks produced)`
    pub by_identity: HashMap<String, (usize, usize)>,
    pub range: RpcBlockProductionRange,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub struct RpcVersionInfo {
    /// The current version of solana-core
    pub solana_core: String,
    /// first 4 bytes of the FeatureSet identifier
    pub feature_set: Option<u32>,
    // first 4 bytes of the sha1 commit hash
    #[serde(serialize_with = "to_hex", deserialize_with = "from_hex")] // TODO should I do this?
    pub commit: Option<u32>,
    /// Client id
    pub client_id: Option<ClientId>, // TODO maybe rename to client?
}

impl fmt::Debug for RpcVersionInfo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.solana_core)
    }
}

impl fmt::Display for RpcVersionInfo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(version) = self.solana_core.split_whitespace().next() {
            // Display just the semver if possible
            write!(f, "{version}")
        } else {
            write!(f, "{}", self.solana_core)
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub struct RpcIdentity {
    /// The current node identity pubkey
    pub identity: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RpcVote {
    /// Vote account address, as base-58 encoded string
    pub vote_pubkey: String,
    pub slots: Vec<Slot>,
    pub hash: String,
    pub timestamp: Option<UnixTimestamp>,
    pub signature: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RpcVoteAccountStatus {
    pub current: Vec<RpcVoteAccountInfo>,
    pub delinquent: Vec<RpcVoteAccountInfo>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RpcVoteAccountInfo {
    /// Vote account address, as base-58 encoded string
    pub vote_pubkey: String,

    /// The validator identity, as base-58 encoded string
    pub node_pubkey: String,

    /// The current stake, in lamports, delegated to this vote account
    pub activated_stake: u64,

    /// An 8-bit integer used as a fraction (commission/MAX_U8) for rewards payout
    pub commission: u8,

    /// Whether this account is staked for the current epoch
    pub epoch_vote_account: bool,

    /// Latest history of earned credits for up to `MAX_RPC_VOTE_ACCOUNT_INFO_EPOCH_CREDITS_HISTORY` epochs
    ///   each tuple is (Epoch, credits, prev_credits)
    pub epoch_credits: Vec<(Epoch, u64, u64)>,

    /// Most recent slot voted on by this vote account (0 if no votes exist)
    pub last_vote: u64,

    /// Current root slot for this vote account (0 if no root slot exists)
    pub root_slot: Slot,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RpcSignatureConfirmation {
    pub confirmations: usize,
    pub status: Result<()>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RpcSimulateTransactionResult {
    pub err: Option<TransactionError>,
    pub logs: Option<Vec<String>>,
    pub accounts: Option<Vec<Option<UiAccount>>>,
    pub units_consumed: Option<u64>,
    pub return_data: Option<UiTransactionReturnData>,
    pub inner_instructions: Option<Vec<UiInnerInstructions>>,
    pub replacement_blockhash: Option<RpcBlockhash>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RpcStorageTurn {
    pub blockhash: String,
    pub slot: Slot,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RpcAccountBalance {
    pub address: String,
    pub lamports: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RpcSupply {
    pub total: u64,
    pub circulating: u64,
    pub non_circulating: u64,
    pub non_circulating_accounts: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum StakeActivationState {
    Activating,
    Active,
    Deactivating,
    Inactive,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct RpcTokenAccountBalance {
    pub address: String,
    #[serde(flatten)]
    pub amount: UiTokenAmount,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RpcConfirmedTransactionStatusWithSignature {
    pub signature: String,
    pub slot: Slot,
    pub err: Option<TransactionError>,
    pub memo: Option<String>,
    pub block_time: Option<UnixTimestamp>,
    pub confirmation_status: Option<TransactionConfirmationStatus>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RpcPerfSample {
    pub slot: Slot,
    pub num_transactions: u64,
    pub num_non_vote_transactions: Option<u64>,
    pub num_slots: u64,
    pub sample_period_secs: u16,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RpcInflationReward {
    pub epoch: Epoch,
    pub effective_slot: Slot,
    pub amount: u64,            // lamports
    pub post_balance: u64,      // lamports
    pub commission: Option<u8>, // Vote account commission when the reward was credited
}

#[derive(Clone, Deserialize, Serialize, Debug, Error, Eq, PartialEq)]
pub enum RpcBlockUpdateError {
    #[error("block store error")]
    BlockStoreError,

    #[error("unsupported transaction version ({0})")]
    UnsupportedTransactionVersion(u8),
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RpcBlockUpdate {
    pub slot: Slot,
    pub block: Option<UiConfirmedBlock>,
    pub err: Option<RpcBlockUpdateError>,
}

impl From<ConfirmedTransactionStatusWithSignature> for RpcConfirmedTransactionStatusWithSignature {
    fn from(value: ConfirmedTransactionStatusWithSignature) -> Self {
        let ConfirmedTransactionStatusWithSignature {
            signature,
            slot,
            err,
            memo,
            block_time,
        } = value;
        Self {
            signature: signature.to_string(),
            slot,
            err,
            memo,
            block_time,
            confirmation_status: None,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub struct RpcSnapshotSlotInfo {
    pub full: Slot,
    pub incremental: Option<Slot>,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RpcPrioritizationFee {
    pub slot: Slot,
    pub prioritization_fee: u64,
}

#[cfg(test)]
pub mod tests {

    use {super::*, serde_json::json};


    #[test]
    fn test_rpc_contact_info_serialization() {
        pub const PUBKEY: &str = "7RoSF9fUmdphVCpabEoefH81WwrW7orsWonXWqTXkKV8";
        let foo = serde_json::to_value(vec![RpcContactInfo {
            pubkey: PUBKEY.to_string(),
            gossip: Some(SocketAddr::from(([10, 239, 6, 48], 8899))),
            tvu: Some(SocketAddr::from(([10, 239, 6, 48], 8865))),
            tpu: Some(SocketAddr::from(([10, 239, 6, 48], 8856))),
            tpu_quic: Some(SocketAddr::from(([10, 239, 6, 48], 8862))),
            tpu_forwards: Some(SocketAddr::from(([10, 239, 6, 48], 8857))),
            tpu_forwards_quic: Some(SocketAddr::from(([10, 239, 6, 48], 8863))),
            tpu_vote: Some(SocketAddr::from(([10, 239, 6, 48], 8870))),
            serve_repair: Some(SocketAddr::from(([10, 239, 6, 48], 8880))),
            rpc: Some(SocketAddr::from(([10, 239, 6, 48], 8899))),
            pubsub: Some(SocketAddr::from(([10, 239, 6, 48], 8900))),
            version: Some("1.0.0 c375ce1f".to_string()),
            feature_set: None,
            shred_version: None,
            commit: Some(123456),
            client_id: Some(ClientId::Agave),
        }]).unwrap();
        println!("foo: {:?}", foo);
    }
    #[test]
    fn test_rpc_contact_info_deserialization() {
        pub const PUBKEY: &str = "7RoSF9fUmdphVCpabEoefH81WwrW7orsWonXWqTXkKV8";
        // let version = solana_version::Version::default();
        // let json = json!({
        //     "pubkey": PUBKEY.to_string(),
        //     "gossip": "127.0.0.1:8000",
        //     "shredVersion": 0u16,
        //     "tvu": "127.0.0.1:8001",
        //     "tpu": "127.0.0.1:8003",
        //     "tpuQuic": "127.0.0.1:8009",
        //     "tpuForwards": "127.0.0.1:8004",
        //     "tpuForwardsQuic": "127.0.0.1:8010",
        //     "tpuVote": "127.0.0.1:8005",
        //     "serveRepair": "127.0.0.1:8008",
        //     "rpc": "127.0.0.1:8123",
        //     "pubsub": "127.0.0.1:8124",
        //     "version": format!("{version}"),
        //     "featureSet": version.feature_set,
        //     "commit": "b2691dc3",
        //     "clientId": "Agave",
        // });
        // println!("json: {:?}", json);
        let value = serde_json::to_value(RpcContactInfo {
            pubkey: PUBKEY.to_string(),
            gossip: Some(SocketAddr::from(([10, 239, 6, 48], 8899))),
            tvu: Some(SocketAddr::from(([10, 239, 6, 48], 8865))),
            tpu: Some(SocketAddr::from(([10, 239, 6, 48], 8856))),
            tpu_quic: Some(SocketAddr::from(([10, 239, 6, 48], 8862))),
            tpu_forwards: Some(SocketAddr::from(([10, 239, 6, 48], 8857))),
            tpu_forwards_quic: Some(SocketAddr::from(([10, 239, 6, 48], 8863))),
            tpu_vote: Some(SocketAddr::from(([10, 239, 6, 48], 8870))),
            serve_repair: Some(SocketAddr::from(([10, 239, 6, 48], 8880))),
            rpc: Some(SocketAddr::from(([10, 239, 6, 48], 8899))),
            pubsub: Some(SocketAddr::from(([10, 239, 6, 48], 8900))),
            version: Some("1.0.0 c375ce1f".to_string()),
            feature_set: None,
            shred_version: None,
            commit: Some(123456),
            client_id: Some(ClientId::Agave),
        }).unwrap();
        println!("value: {:?}", value);
        let info: RpcContactInfo = serde_json::from_value(value).unwrap();
        println!("info: {:?}", info);

    }

    // Make sure that `RpcPerfSample` can read previous version JSON, one without the
    // `num_non_vote_transactions` field.
    #[test]
    fn rpc_perf_sample_deserialize_old() {
        let slot = 424;
        let num_transactions = 2597;
        let num_slots = 2783;
        let sample_period_secs = 398;

        let input = json!({
            "slot": slot,
            "numTransactions": num_transactions,
            "numSlots": num_slots,
            "samplePeriodSecs": sample_period_secs,
        })
        .to_string();

        let actual: RpcPerfSample =
            serde_json::from_str(&input).expect("Can parse RpcPerfSample from string as JSON");
        let expected = RpcPerfSample {
            slot,
            num_transactions,
            num_non_vote_transactions: None,
            num_slots,
            sample_period_secs,
        };

        assert_eq!(actual, expected);
    }

    // Make sure that `RpcPerfSample` serializes into the new `num_non_vote_transactions` field.
    #[test]
    fn rpc_perf_sample_serializes_num_non_vote_transactions() {
        let slot = 1286;
        let num_transactions = 1732;
        let num_non_vote_transactions = Some(757);
        let num_slots = 393;
        let sample_period_secs = 197;

        let input = RpcPerfSample {
            slot,
            num_transactions,
            num_non_vote_transactions,
            num_slots,
            sample_period_secs,
        };
        let actual =
            serde_json::to_value(input).expect("Can convert RpcPerfSample into a JSON value");
        let expected = json!({
            "slot": slot,
            "numTransactions": num_transactions,
            "numNonVoteTransactions": num_non_vote_transactions,
            "numSlots": num_slots,
            "samplePeriodSecs": sample_period_secs,
        });

        assert_eq!(actual, expected);
    }
}
