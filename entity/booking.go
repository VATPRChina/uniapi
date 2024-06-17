package entity

import "github.com/oklog/ulid/v2"

type Booking struct {
	ModelBase
	SlotId ulid.ULID `gorm:"type:uuid;not null;unique"`
}
