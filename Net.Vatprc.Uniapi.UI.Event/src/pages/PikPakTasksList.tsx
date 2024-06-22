import api from "../api";
import { Edit } from "../components/pikpak/Edit";
import { Import } from "../components/pikpak/Import";
import { ErrorTip } from "../components/utils/ErrorTip";
import {
  HStack,
  Heading,
  Icon,
  Link,
  Stack,
  Table,
  TableContainer,
  Tbody,
  Td,
  Text,
  Th,
  Thead,
  Tr,
} from "@chakra-ui/react";
import React from "react";
import { IoCloudOfflineOutline, IoSyncOutline } from "react-icons/io5";

export const PikPakTasksList: React.FunctionComponent = () => {
  const { data: tasks, error } = api.usePikPakList();
  return (
    <>
      <Heading as="h1" size="xl" color="green.700">
        Tasks
      </Heading>
      <Stack spacing="1">
        <ErrorTip error={error} />
        <HStack spacing="1">
          <Edit />
          <Import />
        </HStack>
        <TableContainer>
          <Table variant="simple">
            <Thead>
              <Tr>
                <Th>ID</Th>
                <Th>Bangumi</Th>
                <Th>Target Folder</Th>
                <Th>RegEx</Th>
                <Th></Th>
              </Tr>
            </Thead>
            <Tbody>
              {tasks?.flat()?.map((t) => (
                <Tr key={t.id}>
                  <Td>
                    <HStack>
                      <Text>{t.id}</Text>
                      {t.enabled ? (
                        <Icon as={IoSyncOutline} color="green.500" />
                      ) : (
                        <Icon as={IoCloudOfflineOutline} color="orange.500" />
                      )}
                    </HStack>
                  </Td>
                  <Td>
                    <Link color="teal.500" href={`https://bgm.tv/subject/${t.bangumi}`} target="_blank">
                      {t.bangumi}
                    </Link>
                  </Td>
                  <Td>
                    <Text whiteSpace="normal">{t.target}</Text>
                  </Td>
                  <Td>
                    <Text whiteSpace="normal">{t.regex}</Text>
                  </Td>
                  <Td>
                    <Edit id={t.id} />
                  </Td>
                </Tr>
              ))}
            </Tbody>
          </Table>
        </TableContainer>
      </Stack>
    </>
  );
};
