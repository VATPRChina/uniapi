package entity

import "github.com/oklog/ulid/v2"

type EventAirspace struct {
	ModelBase
	EventId ulid.ULID `gorm:"type:uuid;not null"`
	Ident   string    `gorm:"size:12;not null;unique"`
	Name    string    `gorm:"size:256;not null"`

	Event Event
}
