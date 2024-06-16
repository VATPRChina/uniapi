package external

import (
	"fmt"
	"os"

	"github.com/go-resty/resty/v2"
)

type GroupList struct {
	Groups []struct {
		ID          int    `json:"id"`
		Name        string `json:"name"`
		DisplayName string `json:"display_name,omitempty"`
	} `json:"groups"`
	TotalRowsGroups int    `json:"total_rows_groups"`
	LoadMoreGroups  string `json:"load_more_groups"`
}

type GroupMembersList struct {
	Members []struct {
		ID       int    `json:"id"`
		Username string `json:"username"`
		Name     string `json:"name"`
	} `json:"members"`
}

type User struct {
	UserBadges []interface{} `json:"user_badges"`
	User       struct {
		ID       int    `json:"id"`
		Username string `json:"username"`
		Name     string `json:"name"`
	} `json:"user"`
}

var discourseClient = resty.New()

func GetATCGroupMembers() (*GroupMembersList, error) {
	resp, err := discourseClient.R().
		SetResult(new(GroupMembersList)).
		SetHeader("Api-Key", os.Getenv("DISCOURSE_API_KEY")).
		SetHeader("Api-Username", os.Getenv("DISCOURSE_API_USERNAME")).
		SetPathParam("id", "ATC").
		SetQueryParam("limit", "1000").
		Get("https://community.vatprc.net/groups/{id}/members.json")
	if err != nil {
		fmt.Print(err.Error())
		os.Exit(1)
	}

	return resp.Result().(*GroupMembersList), err
}

func GetUser(id int) (*resty.Response, *User, error) {
	resp, err := discourseClient.R().
		SetResult(new(User)).
		SetHeader("Api-Key", os.Getenv("DISCOURSE_API_KEY")).
		SetHeader("Api-Username", os.Getenv("DISCOURSE_API_USERNAME")).
		SetPathParam("username", fmt.Sprintf("%d", id)).
		Put("https://community.vatprc.net/u/{username}.json")
	if err != nil {
		fmt.Print(err.Error())
		os.Exit(1)
	}

	return resp, resp.Result().(*User), err
}

func AddMember(insertListStr string) (*resty.Response, error) {
	resp, err := discourseClient.R().
		SetResult(new(User)).
		SetHeader("Api-Key", os.Getenv("DISCOURSE_API_KEY")).
		SetHeader("Api-Username", os.Getenv("DISCOURSE_API_USERNAME")).
		SetPathParam("id", "41").
		SetBody(map[string]string{
			"usernames": insertListStr,
		}).
		Put("https://community.vatprc.net/groups/{id}/members.json")
	if err != nil {
		fmt.Print(err.Error())
		os.Exit(1)
	}
	return resp, err
}

func RemoveMember(removeListStr string) (*resty.Response, error) {
	resp, err := discourseClient.R().
		SetResult(new(User)).
		SetHeader("Api-Key", os.Getenv("DISCOURSE_API_KEY")).
		SetHeader("Api-Username", os.Getenv("DISCOURSE_API_USERNAME")).
		SetPathParam("id", "41").
		SetBody(map[string]string{
			"usernames": removeListStr,
		}).
		Delete("https://community.vatprc.net/groups/{id}/members.json")
	if err != nil {
		fmt.Print(err.Error())
		os.Exit(1)
	}
	return resp, err
}
