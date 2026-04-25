use std::{fmt, str::FromStr};

use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UserRole {
    Staff,
    Volunteer,
    DivisionDirector,
    ControllerTrainingDirector,
    ControllerTrainingDirectorAssistant,
    ControllerTrainingInstructor,
    ControllerTrainingMentor,
    ControllerTrainingSopEditor,
    CommunityDirector,
    OperationDirector,
    OperationDirectorAssistant,
    OperationSectorEditor,
    OperationLoaEditor,
    EventDirector,
    LeadEventCoordinator,
    EventCoordinator,
    EventGraphicsDesigner,
    TechDirector,
    TechDirectorAssistant,
    TechAfvFacilityEngineer,
    Controller,
    ApiClient,
    User,
}

impl UserRole {
    pub const fn as_str(self) -> &'static str {
        match self {
            UserRole::Staff => "staff",
            UserRole::Volunteer => "volunteer",
            UserRole::DivisionDirector => "director",
            UserRole::ControllerTrainingDirector => "controller-training-director",
            UserRole::ControllerTrainingDirectorAssistant => {
                "controller-training-director-assistant"
            }
            UserRole::ControllerTrainingInstructor => "controller-training-instructor",
            UserRole::ControllerTrainingMentor => "controller-training-mentor",
            UserRole::ControllerTrainingSopEditor => "controller-training-sop-editor",
            UserRole::CommunityDirector => "community-director",
            UserRole::OperationDirector => "operation-director",
            UserRole::OperationDirectorAssistant => "operation-director-assistant",
            UserRole::OperationSectorEditor => "operation-sector-editor",
            UserRole::OperationLoaEditor => "operation-loa-editor",
            UserRole::EventDirector => "event-director",
            UserRole::LeadEventCoordinator => "lead-event-coordinator",
            UserRole::EventCoordinator => "event-coordinator",
            UserRole::EventGraphicsDesigner => "event-graphics-designer",
            UserRole::TechDirector => "tech-director",
            UserRole::TechDirectorAssistant => "tech-director-assistant",
            UserRole::TechAfvFacilityEngineer => "tech-afv-facility-engineer",
            UserRole::Controller => "controller",
            UserRole::ApiClient => "api-client",
            UserRole::User => "user",
        }
    }
}

impl fmt::Display for UserRole {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl From<UserRole> for &'static str {
    fn from(role: UserRole) -> Self {
        role.as_str()
    }
}

impl From<UserRole> for String {
    fn from(role: UserRole) -> Self {
        role.as_str().to_string()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ParseUserRoleError {
    #[error("Invalid string {0}")]
    InvalidString(String),
}

impl FromStr for UserRole {
    type Err = ParseUserRoleError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "staff" => Ok(UserRole::Staff),
            "volunteer" => Ok(UserRole::Volunteer),
            "director" => Ok(UserRole::DivisionDirector),
            "controller-training-director" => Ok(UserRole::ControllerTrainingDirector),
            "controller-training-director-assistant" => {
                Ok(UserRole::ControllerTrainingDirectorAssistant)
            }
            "controller-training-instructor" => Ok(UserRole::ControllerTrainingInstructor),
            "controller-training-mentor" => Ok(UserRole::ControllerTrainingMentor),
            "controller-training-sop-editor" => Ok(UserRole::ControllerTrainingSopEditor),
            "community-director" => Ok(UserRole::CommunityDirector),
            "operation-director" => Ok(UserRole::OperationDirector),
            "operation-director-assistant" => Ok(UserRole::OperationDirectorAssistant),
            "operation-sector-editor" => Ok(UserRole::OperationSectorEditor),
            "operation-loa-editor" => Ok(UserRole::OperationLoaEditor),
            "event-director" => Ok(UserRole::EventDirector),
            "lead-event-coordinator" => Ok(UserRole::LeadEventCoordinator),
            "event-coordinator" => Ok(UserRole::EventCoordinator),
            "event-graphics-designer" => Ok(UserRole::EventGraphicsDesigner),
            "tech-director" => Ok(UserRole::TechDirector),
            "tech-director-assistant" => Ok(UserRole::TechDirectorAssistant),
            "tech-afv-facility-engineer" => Ok(UserRole::TechAfvFacilityEngineer),
            "controller" => Ok(UserRole::Controller),
            "api-client" => Ok(UserRole::ApiClient),
            "user" => Ok(UserRole::User),
            _ => Err(ParseUserRoleError::InvalidString(value.to_owned())),
        }
    }
}
