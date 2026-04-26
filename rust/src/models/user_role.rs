use std::{collections::HashSet, fmt, str::FromStr};

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

pub fn role_closure(roles: impl IntoIterator<Item = UserRole>) -> HashSet<UserRole> {
    let mut all_roles = HashSet::new();
    let mut stack = roles.into_iter().collect::<Vec<_>>();

    while let Some(role) = stack.pop() {
        if !all_roles.insert(role) {
            continue;
        }

        stack.extend(implied_roles(role));
    }

    all_roles
}

pub fn role_closure_from_strings<'a>(
    roles: impl IntoIterator<Item = &'a str>,
) -> HashSet<UserRole> {
    role_closure(
        roles
            .into_iter()
            .filter_map(|role| role.parse::<UserRole>().ok()),
    )
}

pub fn implied_roles(role: UserRole) -> &'static [UserRole] {
    match role {
        UserRole::Staff => &[UserRole::Volunteer],
        UserRole::DivisionDirector => &[
            UserRole::Staff,
            UserRole::ControllerTrainingDirector,
            UserRole::OperationDirector,
            UserRole::EventDirector,
            UserRole::TechDirector,
        ],
        UserRole::ControllerTrainingDirector => &[
            UserRole::Staff,
            UserRole::ControllerTrainingDirectorAssistant,
            UserRole::ControllerTrainingInstructor,
            UserRole::ControllerTrainingMentor,
            UserRole::ControllerTrainingSopEditor,
        ],
        UserRole::ControllerTrainingDirectorAssistant => &[UserRole::Volunteer],
        UserRole::ControllerTrainingInstructor => {
            &[UserRole::Volunteer, UserRole::ControllerTrainingMentor]
        }
        UserRole::ControllerTrainingMentor => &[UserRole::Volunteer],
        UserRole::ControllerTrainingSopEditor => &[UserRole::Volunteer],
        UserRole::CommunityDirector => &[UserRole::Staff],
        UserRole::OperationDirector => &[
            UserRole::Staff,
            UserRole::OperationDirectorAssistant,
            UserRole::OperationSectorEditor,
            UserRole::OperationLoaEditor,
        ],
        UserRole::OperationDirectorAssistant => &[UserRole::Volunteer],
        UserRole::OperationSectorEditor => &[UserRole::Volunteer],
        UserRole::OperationLoaEditor => &[UserRole::Volunteer],
        UserRole::EventDirector => &[
            UserRole::Staff,
            UserRole::EventCoordinator,
            UserRole::LeadEventCoordinator,
            UserRole::EventGraphicsDesigner,
        ],
        UserRole::LeadEventCoordinator => &[UserRole::EventCoordinator],
        UserRole::EventCoordinator => &[UserRole::Volunteer],
        UserRole::EventGraphicsDesigner => &[UserRole::Volunteer],
        UserRole::TechDirector => &[
            UserRole::Staff,
            UserRole::TechDirectorAssistant,
            UserRole::TechAfvFacilityEngineer,
        ],
        UserRole::TechDirectorAssistant => &[UserRole::Volunteer],
        UserRole::TechAfvFacilityEngineer => &[UserRole::Volunteer],
        _ => &[],
    }
}
