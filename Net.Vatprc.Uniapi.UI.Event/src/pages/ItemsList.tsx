import api from "../api";
import { ImportBangumi } from "../components/anime/ImportBangumi";
import { ErrorTip } from "../components/utils/ErrorTip";
import {
  Box,
  Button,
  GridItem,
  HStack,
  Heading,
  Icon,
  Image,
  SimpleGrid,
  Stack,
  Tooltip,
  VStack,
} from "@chakra-ui/react";
import React from "react";
import { IoImageSharp } from "react-icons/io5";

export const ItemsList: React.FunctionComponent = () => {
  const { data: animes, error } = api.useItemList();

  return (
    <>
      <Heading as="h1" size="xl" color="green.700">
        Animations
      </Heading>
      <Stack spacing="1">
        <ErrorTip error={error} />
        <HStack spacing="1.5">
          <Button size="sm">Today</Button>
          <Button size="sm">3 Days</Button>
          <ImportBangumi />
        </HStack>
        <Box>
          <SimpleGrid
            spacingX="3"
            spacingY="2"
            alignItems="stretch"
            gridTemplateColumns="repeat(auto-fit, minmax(24ch, 32ch))"
          >
            {animes?.map((a) => (
              <GridItem
                key={a.id}
                px="3"
                py="2"
                borderWidth="1px"
                rounded="md"
                bg="white"
                as="a"
                href={`/animes/${a.id}`}
              >
                <VStack height="100%" justifyContent="space-between">
                  {a.image_url ? (
                    <Image src={a.image_url} alt={a.title} flexGrow="1" objectFit="contain" />
                  ) : (
                    <Icon as={IoImageSharp} w="12" h="12" flexGrow="1" />
                  )}
                  <Tooltip label={a.title}>
                    <Heading as="h3" size="sm" noOfLines={1}>
                      {a.title}
                    </Heading>
                  </Tooltip>
                </VStack>
              </GridItem>
            ))}
          </SimpleGrid>
        </Box>
      </Stack>
    </>
  );
};
