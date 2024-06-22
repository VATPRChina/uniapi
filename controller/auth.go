package controller

import (
	"context"

	"github.com/danielgtaylor/huma/v2"
	"github.com/vatprchina/uniapi/usecase"
)

type authorizeInput struct {
	Body struct {
		GrantType string `json:"grant_type" example:"password" doc:"Must be password"`
		Username  string `json:"username" example:"10000001" doc:"Username"`
		Password  string `json:"password" example:"foobar" doc:"Type any value"`
	}
}

type authorizeOutput struct {
	Body struct {
		AccessToken  string `json:"access_token" doc:""`
		TokenType    string `json:"token_type" doc:"Bearer"`
		ExpiresIn    int    `json:"expires_in" doc:"the duration of time the access token is granted for, in seconds"`
		RefreshToken string `json:"refresh_token,omitempty" doc:""`
	}
}

func greet(ctx context.Context, input *authorizeInput) (*authorizeOutput, error) {
	ss, duration, err := usecase.AuthUsecase.IssueAccessToken(input.Body.Username)
	if err != nil {
		return nil, err
	}
	resp := &authorizeOutput{}
	resp.Body.AccessToken = *ss
	resp.Body.TokenType = "Bearer"
	resp.Body.ExpiresIn = int(duration.Seconds())
	return resp, nil
}

func AddAuthRoutes(api huma.API) {
	huma.Post(api, "/oauth/authorize", greet)
}
