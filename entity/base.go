package entity

import (
	"time"

	"github.com/oklog/ulid/v2"
	"gorm.io/gorm"
)

type ModelBase struct {
	ID        ulid.ULID      `gorm:"primarykey;type:uuid"`
	CreatedAt time.Time      `gorm:"not null"`
	UpdatedAt time.Time      `gorm:"not null"`
	DeletedAt gorm.DeletedAt `gorm:"index"`
}

func (b *ModelBase) BeforeCreate(tx *gorm.DB) (err error) {
	b.ID = ulid.Make()

	return
}
