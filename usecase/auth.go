package usecase

import (
	"time"

	"github.com/golang-jwt/jwt/v5"
	"github.com/vatprchina/uniapi/util"
)

func IssueAccessToken(cid string) (*string, *time.Duration, error) {
	duration, _ := time.ParseDuration("1h")
	claims := &jwt.RegisteredClaims{
		ExpiresAt: jwt.NewNumericDate(time.Now().Add(duration)),
		Issuer:    "test",
	}

	token := jwt.NewWithClaims(jwt.SigningMethodES256, claims)
	ss, err := token.SignedString(util.Config.Jwt.PrivateKeyParsed)
	if err != nil {
		return nil, nil, err
	}

	return &ss, &duration, nil
}
