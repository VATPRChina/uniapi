import api, { ApiError, TorrentDto } from "../api";
import { ErrorTip } from "../components/utils/ErrorTip";
import { PaseAniResult, PaseAniTagType, TAG_TYPE_TO_COLOR } from "../services/paseani";
import {
  Button,
  HStack,
  Heading,
  Icon,
  IconButton,
  Input,
  InputGroup,
  InputLeftElement,
  Progress,
  Stack,
  Tag,
  TagLeftIcon,
  Tooltip,
} from "@chakra-ui/react";
import React, { useState } from "react";
import { IoDownloadOutline, IoInformation, IoLink, IoMagnetOutline, IoSearchOutline } from "react-icons/io5";
import useSWR from "swr";
import useSWRInfinite from "swr/infinite";
import { useCopyToClipboard, useDebounce } from "usehooks-ts";

export const TorrentsList: React.FunctionComponent = () => {
  const [rawQuery, setQuery] = useState("");
  const query = useDebounce(rawQuery, 300);
  // TODO: generate API hooks for this pattern
  const {
    data: torrents,
    error,
    size,
    setSize,
    isLoading,
  } = useSWRInfinite<TorrentDto[], ApiError>(
    (pageIndex, previousPageData: TorrentDto[]): [string, string, string | undefined] | null => {
      if (previousPageData?.length === 0) return null;

      if (pageIndex === 0) return ["/api/modules/torrent_directory/torrents", query, undefined];

      return [
        "/api/modules/torrent_directory/torrents",
        query,
        previousPageData
          ?.map((d) => new Date(d.published_at))
          .sort((a, b) => a.getTime() - b.getTime())[0]
          ?.toISOString() ?? "",
      ];
    },
    {
      fetcher: ([, query, until]: [string, string, string | undefined]): Promise<TorrentDto[]> =>
        api.torrentList({ query: query ?? "", until }).then((d) => d.data),
    },
  );
  const [, copy] = useCopyToClipboard();
  const [titleToParse, setTitleToParse] = useState("");
  const { data: parsedTitle, isLoading: parseLoading } = useSWR<PaseAniResult>(
    titleToParse.length > 0 ? `https://paseani.zhuiani.me/info?name=${encodeURIComponent(titleToParse)}` : null,
  );

  return (
    <>
      <Heading as="h1" size="xl" color="green.700">
        Torrents
      </Heading>
      <Stack spacing="1">
        <ErrorTip error={error} />
        <InputGroup>
          <InputLeftElement pointerEvents="none">
            <Icon color="gray.300" as={IoSearchOutline} />
          </InputLeftElement>
          <Input placeholder="Query (regexp)" onChange={(e) => setQuery(e.target.value)} />
        </InputGroup>
        {isLoading && <Progress isIndeterminate />}
        {torrents?.flat()?.map((a) => (
          <>
            <HStack key={a.id} px="3" py="2" borderWidth="1px" rounded="md" bg="white" gap="2">
              {a.link_torrent && (
                <a href={a.link_torrent} target="_blank" rel="noreferrer">
                  <Tooltip label={a.link_torrent} placement="top">
                    <IconButton
                      variant="solid"
                      colorScheme="teal"
                      size="sm"
                      fontSize="1.125rem"
                      icon={<IoDownloadOutline />}
                      aria-label="Download"
                    />
                  </Tooltip>
                </a>
              )}
              {a.link_magnet && (
                <Tooltip label={a.link_magnet} placement="top">
                  <IconButton
                    variant="solid"
                    colorScheme="teal"
                    size="sm"
                    fontSize="1.125rem"
                    onClick={() => void copy(a.link_magnet ?? "")}
                    icon={<IoMagnetOutline />}
                    aria-label="Copy Magnet link"
                  />
                </Tooltip>
              )}
              <Heading as="h3" size="sm" noOfLines={1}>
                {a.title}
              </Heading>
              <Tooltip label="Powered by PaseAni" placement="top">
                <IconButton
                  variant="ghost"
                  colorScheme="teal"
                  size="sm"
                  fontSize="1.125rem"
                  onClick={() => setTitleToParse(a.title)}
                  icon={<IoInformation />}
                  aria-label="Parse"
                  isLoading={titleToParse === a.title && parseLoading}
                />
              </Tooltip>
            </HStack>
            {titleToParse === a.title && parsedTitle && (
              <HStack>
                {parsedTitle.tags.map((t) => (
                  <Tooltip key={t.value} label={`${t.parser} | ${t.type}`}>
                    {t.type === PaseAniTagType.link ? (
                      <a href={t.value} target="_blank" rel="noreferrer">
                        <Tag colorScheme={TAG_TYPE_TO_COLOR[t.type]}>
                          <TagLeftIcon boxSize="12px" as={IoLink} />
                          {new URL(t.value).hostname}
                        </Tag>
                      </a>
                    ) : (
                      <Tag colorScheme={TAG_TYPE_TO_COLOR[t.type]}>{t.value}</Tag>
                    )}
                  </Tooltip>
                ))}
              </HStack>
            )}
          </>
        ))}
        <Button
          variant="solid"
          colorScheme="teal"
          size="xs"
          onClick={() => void setSize(size + 1)}
          disabled={isLoading}
        >
          Load More
        </Button>
      </Stack>
    </>
  );
};
