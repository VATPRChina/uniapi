package usecase

import (
	"time"

	"github.com/golang-jwt/jwt/v5"
	"github.com/vatprchina/uniapi/util"
)

type authUsecase struct{}

var Auth = &authUsecase{}

func (*authUsecase) IssueAccessToken(cid string) (*string, *time.Duration, error) {
	claims := &jwt.RegisteredClaims{
		ExpiresAt: jwt.NewNumericDate(time.Now().Add(util.Config.Jwt.Duration)),
		Issuer:    util.Config.Jwt.Issuer,
		Subject:   cid,
		NotBefore: jwt.NewNumericDate(time.Now()),
		IssuedAt:  jwt.NewNumericDate(time.Now()),
	}

	token := jwt.NewWithClaims(jwt.SigningMethodES256, claims)
	ss, err := token.SignedString(util.Config.Jwt.PrivateKeyParsed)
	if err != nil {
		return nil, nil, err
	}

	return &ss, &util.Config.Jwt.Duration, nil
}
