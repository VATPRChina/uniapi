import api from "../../api";
import { promiseWithToast, toast } from "../../utils";
import {
  Box,
  Button,
  HStack,
  Icon,
  IconButton,
  Input,
  Popover,
  PopoverBody,
  PopoverContent,
  PopoverTrigger,
} from "@chakra-ui/react";
import React, { useState } from "react";
import { IoCloseOutline } from "react-icons/io5";
import { LuImport } from "react-icons/lu";

export const ImportBangumi: React.FunctionComponent = () => {
  const [isOpen, setIsOpen] = useState(false);
  const [importLink, setImportLink] = useState("");

  const onOpenClick = () => setIsOpen(true);
  const onCloseClick = () => setIsOpen(false);

  const { trigger, isMutating } = api.useBangumiImportSubject();

  const onImportClick = () =>
    promiseWithToast(
      async () => {
        const subject = Number.parseInt(importLink.replace("https://bgm.tv/subject/", ""), 10);
        await trigger({ id: subject });
        toast({
          title: `Subject ${subject} imported.`,
          status: "success",
          duration: 5000,
          isClosable: true,
        });
      },
      () => setIsOpen(false),
    );

  return (
    <Box>
      <Popover isOpen={isOpen}>
        <PopoverTrigger>
          <Button size="sm" onClick={onOpenClick}>
            Import Anime from Bangumi
          </Button>
        </PopoverTrigger>
        <PopoverContent>
          <PopoverBody>
            <HStack>
              <Input placeholder="Link" value={importLink} onChange={(e) => setImportLink(e.target.value)} />
              <IconButton
                aria-label="Import"
                icon={<Icon as={LuImport} />}
                onClick={onImportClick}
                isLoading={isMutating}
              />
              <IconButton
                aria-label="Cancel"
                icon={<Icon as={IoCloseOutline} />}
                onClick={onCloseClick}
                isDisabled={isMutating}
              />
            </HStack>
          </PopoverBody>
        </PopoverContent>
      </Popover>
    </Box>
  );
};
