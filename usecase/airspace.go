package usecase

import (
	"github.com/vatprchina/uniapi/entity"
	"github.com/vatprchina/uniapi/external"
)

type airspaceUsecase struct{}

var Airspace = &airspaceUsecase{}

func (svc *airspaceUsecase) Create(ident string, name string) {
	airspace := &entity.EventAirspace{
		Ident: ident,
		Name:  name,
	}
	result := external.Database.Create(airspace)
	if result.Error != nil {
		panic(result.Error)
	}
}
