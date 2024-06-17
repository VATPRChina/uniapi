package entity

type Airspace struct {
	ModelBase
	Ident string `gorm:"size:12;not null;unique"`
	Name  string `gorm:"size:256;not null"`
}
