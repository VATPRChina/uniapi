package main

import (
	"fmt"
	"os"
	"strings"

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

func main() {
	client := resty.New()

	resp, err := client.R().SetResult(new(ATCList)).Get("https://atcapi.vatprc.net/v1/public/controllers")
	if err != nil {
		fmt.Print(err.Error())
		os.Exit(1)
	}

	atcList := resp.Result().(*ATCList)

	resp, err = client.R().
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

	insertList := []string{}
	removeList := []string{}

	atcGroupMembers := resp.Result().(*GroupMembersList)
	for _, member := range atcGroupMembers.Members {
		fmt.Printf("ATC group User %s (%d, %s)\n", member.Name, member.ID, member.Username)
		found := false
		for _, atc := range *atcList {
			if fmt.Sprintf("%d", atc.ID) == member.Username {
				found = true
				break
			}
		}
		if !found {
			removeList = append(removeList, member.Username)
			fmt.Printf("Remove from ATC group User %s (%d, %s)\n", member.Name, member.ID, member.Username)
		}
	}
	for _, atc := range *atcList {
		found := false
		for _, member := range atcGroupMembers.Members {
			if fmt.Sprintf("%d", atc.ID) == member.Username {
				found = true
				break
			}
		}
		if !found {
			resp, err := client.R().
				SetResult(new(User)).
				SetHeader("Api-Key", os.Getenv("DISCOURSE_API_KEY")).
				SetHeader("Api-Username", os.Getenv("DISCOURSE_API_USERNAME")).
				SetPathParam("username", fmt.Sprintf("%d", atc.ID)).
				Put("https://community.vatprc.net/u/{username}.json")
			if err != nil {
				fmt.Print(err.Error())
				os.Exit(1)
			}

			if resp.Status() != "200 OK" {
				fmt.Printf("Skip non-exist user %s %s (%d)\n", atc.FirstName, atc.LastName, atc.ID)
				continue
			}
			insertList = append(insertList, fmt.Sprintf("%d", atc.ID))
			fmt.Printf("Insert to ATC group User %s %s (%d)\n", atc.FirstName, atc.LastName, atc.ID)
		}
	}

	insertListStr := strings.Join(insertList, ",")
	resp, err = client.R().
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
	fmt.Printf("Added %v get %s (%s)\n", insertListStr, resp.Status(), resp.String())

	removeListStr := strings.Join(removeList, ",")
	resp, err = client.R().
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
	fmt.Printf("Removed %v get %s (%s)\n", removeListStr, resp.Status(), resp.String())
}
