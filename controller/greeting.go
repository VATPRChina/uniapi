package controller

import (
	"context"
	"fmt"

	"github.com/danielgtaylor/huma/v2"
)

type greetingInput struct {
	Name string `path:"name" maxLength:"30" example:"world" doc:"Name to greet"`
}

type greetingOutput struct {
	Body struct {
		Message string `json:"message" example:"Hello, world!" doc:"Greeting message"`
	}
}

func greet(ctx context.Context, input *greetingInput) (*greetingOutput, error) {
	resp := &greetingOutput{}
	resp.Body.Message = fmt.Sprintf("Hello, %s!", input.Name)
	return resp, nil
}

func AddGreet(api huma.API) {
	huma.Get(api, "/greeting/{name}", greet)
}
