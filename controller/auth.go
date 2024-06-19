package controller

import (
	"context"

	"github.com/danielgtaylor/huma/v2"
)

type greetingInput struct {
	Body struct {
		GrantType string `path:"grant_type" example:"password" doc:"Must be password"`
		Username  string `path:"username" example:"10000001" doc:"Username"`
		Password  string `path:"password" example:"foobar" doc:"Type any value"`
	}
}

type greetingOutput struct {
	Body struct {
		Message string `json:"message" example:"Hello, world!" doc:"Greeting message"`
	}
}

func greet(ctx context.Context, input *greetingInput) (*greetingOutput, error) {
	resp := &greetingOutput{}
	return resp, nil
}

func AddAuthRoutes(api huma.API) {
	huma.Post(api, "/oauth/authorize", greet)
}
