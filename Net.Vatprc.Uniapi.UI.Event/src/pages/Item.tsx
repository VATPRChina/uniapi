import api from "../api";
import { ItemLinks } from "../components/anime/ItemLinks";
import { ErrorTip } from "../components/utils/ErrorTip";
import { OverflowTooltip } from "../components/utils/OverflowTooltip";
import { NullULID } from "../components/utils/misc";
import { Card, CardBody, Flex, Heading, Image, Stack } from "@chakra-ui/react";
import React from "react";
import { useParams } from "react-router-dom";

export const Item: React.FunctionComponent = () => {
  const params = useParams();
  const { data: anime, error: animeError } = api.useItemGet(params["animeId"] ?? NullULID);
  const { data: episodes, error: episodesError } = api.useItemGetChildItems(params["animeId"] ?? NullULID);
  const error = animeError ?? episodesError;

  return (
    <>
      <Heading as="h1" size="xl" color="green.700">
        {anime?.title}
      </Heading>
      <Stack direction={["column", null, null, null, "row"]}>
        {anime && <Image src={anime.image_url ?? ""} alt={anime.title} maxW="32ch" height="max-content" />}
        <Stack spacing="1">
          <ErrorTip error={error} />
          {anime && <ItemLinks id={anime?.id} />}
          <Flex wrap="wrap" rowGap="3" columnGap="2">
            {episodes?.map((x) => (
              <Card key={x.id}>
                <CardBody>
                  <Stack spacing="2">
                    <OverflowTooltip
                      size="md"
                      noOfLines={1}
                      maxW="24ch"
                      tooltipProps={{ placement: "top", bg: "gray.100", color: "black" }}
                    >
                      {x.title}
                    </OverflowTooltip>
                    <ItemLinks id={x.id} />
                  </Stack>
                </CardBody>
              </Card>
            ))}
          </Flex>
        </Stack>
      </Stack>
    </>
  );
};
