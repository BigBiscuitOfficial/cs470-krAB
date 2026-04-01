/// Education level attainment
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EducationLevel {
    HighSchool,
    Associates,
    Bachelors,
    Masters,
    PhD,
}

impl EducationLevel {
    pub fn as_key(self) -> &'static str {
        match self {
            Self::HighSchool => "HighSchool",
            Self::Associates => "Associates",
            Self::Bachelors => "Bachelors",
            Self::Masters => "Masters",
            Self::PhD => "PhD",
        }
    }
}

/// Gender identity
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Gender {
    Male,
    Female,
    NonBinary,
}

impl Gender {
    pub fn as_key(self) -> &'static str {
        match self {
            Self::Male => "Male",
            Self::Female => "Female",
            Self::NonBinary => "NonBinary",
        }
    }
}

/// Race and ethnicity categories
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RaceEthnicity {
    White,
    Black,
    Hispanic,
    Asian,
    NativeAmerican,
    MultiRacial,
}

impl RaceEthnicity {
    pub fn as_key(self) -> &'static str {
        match self {
            Self::White => "White",
            Self::Black => "Black",
            Self::Hispanic => "Hispanic",
            Self::Asian => "Asian",
            Self::NativeAmerican => "NativeAmerican",
            Self::MultiRacial => "MultiRacial",
        }
    }
}

/// Geographic region in the US
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Region {
    Northeast,
    West,
    Midwest,
    South,
    Mountain,
}

impl Region {
    pub fn as_key(self) -> &'static str {
        match self {
            Self::Northeast => "Northeast",
            Self::West => "West",
            Self::Midwest => "Midwest",
            Self::South => "South",
            Self::Mountain => "Mountain",
        }
    }
}

/// Self-reported health status
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HealthStatus {
    Excellent,
    Good,
    Fair,
    Poor,
}

impl HealthStatus {
    pub fn as_key(self) -> &'static str {
        match self {
            Self::Excellent => "Excellent",
            Self::Good => "Good",
            Self::Fair => "Fair",
            Self::Poor => "Poor",
        }
    }
}

/// Type of health insurance coverage
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InsuranceType {
    EmployerSponsored,
    ACAMarketplace,
    Medicaid,
    Medicare,
    Uninsured,
}

impl InsuranceType {
    pub fn as_key(self) -> &'static str {
        match self {
            Self::EmployerSponsored => "EmployerSponsored",
            Self::ACAMarketplace => "ACAMarketplace",
            Self::Medicaid => "Medicaid",
            Self::Medicare => "Medicare",
            Self::Uninsured => "Uninsured",
        }
    }
}

/// Career ambition level affecting income growth
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CareerAmbition {
    Stable,
    Moderate,
    Aggressive,
}

impl CareerAmbition {
    pub fn as_key(self) -> &'static str {
        match self {
            Self::Stable => "Stable",
            Self::Moderate => "Moderate",
            Self::Aggressive => "Aggressive",
        }
    }
}
