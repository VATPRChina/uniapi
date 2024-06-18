package external

import (
	"gorm.io/driver/postgres"
	"gorm.io/gorm"

	"github.com/vatprchina/uniapi/entity"
)

var Database *gorm.DB

func DatabaseConnect() error {
	db, err := gorm.Open(postgres.Open("host=localhost user=xfoxfu dbname=uniapi"), &gorm.Config{TranslateError: true})
	if err != nil {
		return err
	}

	Database = db
	err = db.AutoMigrate(
		new(entity.User),
		new(entity.Event),
		new(entity.EventAirspace),
		new(entity.Slot),
		new(entity.Booking),
	)

	if err != nil {
		return err
	}
	return nil
}
