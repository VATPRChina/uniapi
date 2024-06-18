package entity

type User struct {
	ModelBase
	VatsimCid uint32 `gorm:"not null;unique"`
	FullName  string `gorm:"size:256;not null"`
}
