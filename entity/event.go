package entity

import "time"

type Event struct {
	ModelBase
	Title   string    `gorm:"size:256;not null"`
	StartAt time.Time `gorm:"type:timestamptz;not null"`
	EndAt   time.Time `gorm:"type:timestamptz;not null"`
}
