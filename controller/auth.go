package controller

import (
	"context"

	"github.com/danielgtaylor/huma/v2"
	"github.com/vatprchina/uniapi/usecase"
)

type authorizeInput struct {
	Body struct {
		GrantType string `json:"grant_type" example:"password" doc:"must be 'password'"`
		Username  string `json:"username" example:"10000001" doc:"username, which is the same as CID"`
		Password  string `json:"password" example:"foobar" doc:"type any value"`
	}
}

type authorizeOutput struct {
	Body struct {
		AccessToken  string `json:"access_token" doc:"access token"`
		TokenType    string `json:"token_type" doc:"Bearer"`
		ExpiresIn    int    `json:"expires_in" doc:"the duration of time the access token is granted for, in seconds"`
		RefreshToken string `json:"refresh_token,omitempty" doc:"refresh token"`
	}
}

func authorize(ctx context.Context, input *authorizeInput) (*authorizeOutput, error) {
	ss, duration, err := usecase.Auth.IssueAccessToken(input.Body.Username)
	if err != nil {
		return nil, err
	}
	resp := &authorizeOutput{}
	resp.Body.AccessToken = *ss
	resp.Body.TokenType = "Bearer"
	resp.Body.ExpiresIn = int(duration.Seconds())
	return resp, nil
}

type userInfoInput struct {
	Header struct {
		Authorization string `header:"Authorization"`
	}
}

type userInfoOutput struct {
	Body struct {
		Sub    string `json:"sub" doc:"user identifier, which is the same as CID"`
		Name   string `json:"name" doc:"full name"`
		Groups string `json:"groups" doc:"Possible values: 'admin'"`
	}
}

func userInfo(ctx context.Context, input *userInfoInput) (*userInfoOutput, error) {
	return nil, nil
}

func AddAuthRoutes(api huma.API) {
	huma.Post(api, "/oauth/authorize", authorize)
	huma.Post(api, "/oauth/userinfo", userInfo)
}
