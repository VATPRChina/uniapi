package controller

import "github.com/danielgtaylor/huma/v2"

func AddRoutes(api huma.API) {
	AddAuthRoutes(api)
}
