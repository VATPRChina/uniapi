use std::collections::HashSet;

use serde::{Deserialize, Serialize, de::IntoDeserializer};

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    PartialOrd,
    Ord,
    Deserialize,
    Serialize,
    utoipa::ToSchema,
)]
#[serde(rename_all = "kebab-case")]
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

impl std::fmt::Display for UserRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.serialize(f)
    }
}

impl std::str::FromStr for UserRole {
    type Err = serde::de::value::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        UserRole::deserialize(s.into_deserializer())
    }
}

#[cfg(test)]
#[test]
fn test_user_role_serialization() {
    let role = UserRole::ControllerTrainingMentor;
    let serialized = format!("{}", role);
    assert_eq!(serialized, "controller-training-mentor");
}

#[cfg(test)]
#[test]
fn test_user_role_deserialization() {
    use std::str::FromStr;

    let deserialized = UserRole::from_str("controller-training-mentor").unwrap();
    assert_eq!(deserialized, UserRole::ControllerTrainingMentor);
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
