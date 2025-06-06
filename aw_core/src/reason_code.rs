use num_enum::TryFromPrimitive;

#[derive(Debug, PartialEq, Eq, Clone, Copy, TryFromPrimitive)]
#[repr(i32)]
pub enum ReasonCode {
    Success = 0,
    CitizenshipExpired = 1,
    LandLimitExceeded = 2,
    NoSuchCitizen = 3,
    MessageLengthBad = 4,
    LicensePasswordContainsSpace = 5,
    LicensePasswordTooLong = 6,
    LicensePasswordTooShort = 7,
    LicenseRangeTooLarge = 8,
    LicenseRangeTooSmall = 9,
    LicenseUsersTooLarge = 10,
    LicenseUsersTooSmall = 11,
    LicenseContainsInvalidChar = 12,
    InvalidPassword = 13,
    UnableToMailBackNumber = 14,
    LicenseWorldTooShort = 15,
    LicenseWorldTooLong = 16,
    ServerOutOfMemory = 17,
    SdkMustUpgrade = 18, // Conflicts with 58 in public documentation
    InvalidWorld = 20,
    ServerOutdated = 21,
    WorldAlreadyStarted = 22,
    NotWorldOwner = 26, // Not publicly documented
    NoSuchWorld = 27,
    UnableToChangeCitizen = 28, // Not publicly documented
    NotLoggedIn = 31,
    Unauthorized = 32,
    WorldAlreadyExists = 33,
    NoSuchLicense = 34,
    UnableToSendTelegram = 36, // Not publicly documented
    UnableToGetTelegram = 37,  // Not publicly documented
    UnableToSetContact = 38,   // Not publicly documented
    IdentityAlreadyInUse = 39,
    UnableToReportLocation = 40,
    InvalidEmail = 41,
    NoSuchActingCitizen = 42,
    ActingPasswordInvalid = 43,
    UniverseFull = 45,
    BillingTimeout = 46,
    BillingRecvFailed = 47,
    BillingResponseInvalid = 48,
    ImmigrationNotAllowed = 53, // Not publicly documented
    BillingRejected = 55,
    BillingBlocked = 56,
    TooManyWorlds = 57,
    MustUpgrade = 58,
    BotLimitExceeded = 59,
    WorldExpired = 61,
    CitizenDoesNotExpire = 62,
    LicenseStartsWithNumber = 64,
    NoSuchEjection = 66,
    NoSuchSession = 67,
    EjectionExpired = 69,
    ActingCitizenExpired = 70,
    AlreadyStarted = 71,
    WorldRunning = 72,
    WorldNotSet = 73,
    NoSuchCell = 74,
    NoRegistry = 75,
    CantOpenRegistry = 76,
    CitizenDisabled = 77,
    WorldDisabled = 78,
    BetaRequired = 79,
    ActingCitizenDisabled = 80,
    InvalidUserCount = 81,
    TouristAllowed = 84, // Not publicly documented
    TelegramBlocked = 85,
    TelegramTooLong = 86,
    UnableToUpdateTerrain = 88,
    PrivateWorld = 91,
    NoTourists = 92,
    EmailContainsInvalidChar = 100,
    EmailEndsWithBlank = 101,
    //NoSuchObject = 101,
    EmailMissingDot = 102,
    //NotDeleteOwner = 102,
    EmailMissingAt = 103,
    EmailStartsWithBlank = 104,
    EmailTooLong = 105,
    EmailTooShort = 106,
    NameAlreadyUsed = 107,
    NameContainsNonalphanumericChar = 108,
    NameContainsInvalidBlank = 109,
    NameDoesntExist = 110,
    NameEndsWithBlank = 111,
    NameTooLong = 112,
    NameTooShort = 113,
    NameUnused = 114,
    PasswordTooLong = 115,
    PasswordTooShort = 116,
    PasswordWrong = 117,
    UnableToDeleteName = 119,
    UnableToGetCitizen = 120,
    UnableToInsertCitizen = 121,
    UnableToInsertName = 122,
    UnableToPutCitizenCount = 123,
    UnableToDeleteCitizen = 124,
    NumberAlreadyUsed = 126,
    NumberOutOfRange = 127,
    PrivilegePasswordIsTooShort = 128,
    PrivilegePasswordIsTooLong = 129,
    UnableToChangeLicense = 132, // Not publicly documented
    BotgramNotYet = 137,         // Not publicly documented
    NoPort = 139,                // Not publicly documented
    NotChangeOwner = 203,
    CantFindOldElement = 204,
    UnableToChangeAttribute = 210,
    CantChangeOwner = 211,
    Imposter = 212,
    InvalidRequest = 213,
    CantBuildHere = 216,
    JoinRefused = 250,
    TelegramBlockedByPlugin = 251, // Not publicly documented
    Encroaches = 300,
    ObjectTypeInvalid = 301,
    TooManyBytes = 303,
    UnableToStore = 305,
    UnregisteredObject = 306,
    ElementAlreadyExists = 308,
    RestrictedCommand = 309,
    NoBuildRights = 310,
    OutOfBounds = 311,
    RestrictedObject = 313,
    RestrictedArea = 314,
    OutOfMemory = 400,
    NotYet = 401,
    Timeout = 402,
    NullPointer = 403,
    UnableToContactUniverse = 404,
    UnableToContactWorld = 405,
    InvalidWorldName = 406,
    SendFailed = 415,
    ReceiveFailed = 416,
    StreamEmpty = 421,
    StreamMessageTooLong = 422,
    WorldNameTooLong = 423,
    MessageTooLong = 426,
    TooManyResets = 427,
    UnableToCreateSocket = 428,
    UnableToConnect = 429,
    UnableToSetNonblocking = 430,
    CantOpenStream = 434,
    CantWriteStream = 435,
    CantCloseStream = 436,
    NoConnection = 439,
    UnableToInitializeNetwork = 442,
    IncorrectMessageLength = 443,
    NotInitialized = 444,
    NoInstance = 445,
    OutBufferFull = 446,
    InvalidCallback = 447,
    InvalidAttribute = 448,
    TypeMismatch = 449,
    StringTooLong = 450,
    ReadOnly = 451,
    UnableToRegisterResolve = 452,
    InvalidInstance = 453,
    VersionMismatch = 454,
    InBufferFull = 461,
    ProtocolError = 463,
    QueryInProgress = 464,
    WorldFull = 465,
    Ejected = 466,
    NotWelcome = 467,
    UnableToBind = 468,
    UnableToListen = 469,
    UnableToAccept = 470,
    ConnectionLost = 471,
    NoStream = 473,
    NotAvailable = 474,
    OldUniverse = 487,
    OldWorld = 488,
    WorldNotRunning = 489,
    CantResolveUniverseHost = 500,
    InvalidArgument = 505,
    UnableToUpdateCav = 514,
    UnableToDeleteCav = 515,
    NoSuchCav = 516,
    NoCavTemplate = 517,       // Not publicly documented
    UnableToGetContacts = 520, // Not publicly documented
    WorldInstanceAlreadyExists = 521,
    WorldInstanceInvalid = 522,
    PluginNotAvailable = 523,
    ContactAddBlocked = 524, // Not publicly documented
    EmailChangeNotAllowed = 525,
    NameChangeNotAllowed = 526,
    EmailAlreadyUsed = 527,
    EmailNotAllowed = 528,
    WorldRedirect = 529,
    DatabaseError = 600,
    NoDatabase = 601, // Not publicly documented
    ZBufError = 4995,
    ZMemError = 4996,
    ZDataError = 4997,
}

impl ReasonCode {
    pub fn is_err(&self) -> bool {
        *self != Self::Success
    }

    pub fn is_ok(&self) -> bool {
        !self.is_err()
    }
}

impl From<ReasonCode> for i32 {
    fn from(val: ReasonCode) -> Self {
        val as i32
    }
}

impl From<ReasonCode> for u32 {
    fn from(val: ReasonCode) -> Self {
        val as u32
    }
}
