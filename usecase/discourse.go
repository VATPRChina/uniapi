package usecase

import (
	"fmt"
	"strings"

	"github.com/vatprchina/uniapi/external"
)

func SyncDiscourseATCGroup() {

	insertList := []string{}
	removeList := []string{}

	atcList, _ := external.GetAllAtc()
	atcGroupMembers, _ := external.GetATCGroupMembers()
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
			resp, _, _ := external.GetUser(atc.ID)
			if resp.Status() != "200 OK" {
				fmt.Printf("Skip non-exist user %s %s (%d)\n", atc.FirstName, atc.LastName, atc.ID)
				continue
			}
			insertList = append(insertList, fmt.Sprintf("%d", atc.ID))
			fmt.Printf("Insert to ATC group User %s %s (%d)\n", atc.FirstName, atc.LastName, atc.ID)
		}
	}

	insertListStr := strings.Join(insertList, ",")
	resp, _ := external.AddMember(insertListStr)
	fmt.Printf("Added %v get %s (%s)\n", insertListStr, resp.Status(), resp.String())

	removeListStr := strings.Join(removeList, ",")
	resp, _ = external.RemoveMember(removeListStr)
	fmt.Printf("Removed %v get %s (%s)\n", removeListStr, resp.Status(), resp.String())
}
