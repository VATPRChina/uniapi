package entity

import (
	"time"

	"github.com/oklog/ulid/v2"
)

type Slot struct {
	ModelBase
	EventId    ulid.ULID `gorm:"type:uuid;not null"`
	EnterAt    time.Time `gorm:"type:timestamptz;not null"`
	AirspaceId ulid.ULID `gorm:"type:uuid;not null"`
	Booking    Booking
}
