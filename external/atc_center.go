package external

import (
	"fmt"
	"os"

	"github.com/go-resty/resty/v2"
)

type ATCList []struct {
	ID        int    `json:"id"`
	FirstName string `json:"first_name"`
	LastName  string `json:"last_name"`
	Roles     []struct {
		ID   int    `json:"id"`
		Name string `json:"name"`
	} `json:"roles"`
}

var atcClient = resty.New()

func GetAllAtc() (*ATCList, error) {
	resp, err := atcClient.R().SetResult(new(ATCList)).Get("https://atcapi.vatprc.net/v1/public/controllers")
	if err != nil {
		fmt.Print(err.Error())
		os.Exit(1)
	}

	atcList := resp.Result().(*ATCList)
	return atcList, err
}
