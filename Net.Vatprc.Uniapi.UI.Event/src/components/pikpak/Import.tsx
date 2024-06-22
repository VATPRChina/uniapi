import api from "../../api";
import { promiseWithLog, promiseWithToast, toast } from "../../utils";
import {
  Button,
  FormControl,
  FormErrorMessage,
  FormLabel,
  HStack,
  Icon,
  IconButton,
  Input,
  InputGroup,
  InputRightElement,
  List,
  ListIcon,
  ListItem,
  Modal,
  ModalBody,
  ModalCloseButton,
  ModalContent,
  ModalFooter,
  ModalHeader,
  ModalOverlay,
  Text,
  Textarea,
  VStack,
  useDisclosure,
} from "@chakra-ui/react";
import escape from "escape-string-regexp";
import React, { useEffect, useState } from "react";
import { SubmitHandler, useForm } from "react-hook-form";
import { IoCheckmarkOutline, IoCodeWorkingOutline, IoDocumentOutline, IoFilmOutline } from "react-icons/io5";
import { useDebounce } from "usehooks-ts";

type Inputs = {
  bangumi: string;
  path: string;
  regex: string;
  match_group_ep: number;
};

export const Import: React.FunctionComponent<{ id?: string }> = ({ id }) => {
  const { isOpen, onOpen, onClose } = useDisclosure();
  const {
    register,
    handleSubmit,
    formState: { errors: formErrs, isDirty, isValid, isValidating },
    setValue,
    getValues,
    watch,
  } = useForm<Inputs>();
  const onSubmit: SubmitHandler<Inputs> = (inputs) => {
    promiseWithToast(async () => {
      await api.pikPakImportFolder({
        bangumi: Number.parseInt(inputs.bangumi, 10),
        path: inputs.path,
        regex: inputs.regex,
        match_group_ep: inputs.match_group_ep,
      });
      toast({
        title: "Files imported.",
        status: "success",
        duration: 5000,
        isClosable: true,
      });
      onClose();
    });
  };
  const onEscapeRegex = () => {
    setValue("regex", escape(getValues("regex")), { shouldDirty: true });
  };
  const bangumi = watch("bangumi");
  const bangumiDebounced = useDebounce(bangumi, 1000);
  const [bangumiList, setBangumiList] = useState<[string, number][]>([]);
  useEffect(() => {
    promiseWithLog(async () => {
      if (!bangumiDebounced || !isOpen) return;
      const list = await api.bangumiSearchSubject({ query: bangumiDebounced });
      const subject = Number.isNaN(Number.parseInt(bangumiDebounced))
        ? null
        : await api.bangumiGetSubject(Number.parseInt(bangumiDebounced));
      if (subject) list.data.unshift(subject.data);
      setBangumiList(list.data.map((item) => [item.name_cn || item.name, item.id]));
    });
  }, [bangumiDebounced, isOpen]);

  const path = watch("path");
  const pathDebounced = useDebounce(path, 1000);
  const [fileList, setFileList] = useState<string[]>([]);
  useEffect(() => {
    promiseWithLog(async () => {
      if (!pathDebounced || !isOpen) return;
      const list = await api.pikPakListFolder({ path: pathDebounced });
      setFileList(list.data.map((item) => `${pathDebounced}/${item.name}`));
    });
  }, [isOpen, pathDebounced]);

  return (
    <>
      <Button colorScheme="teal" onClick={onOpen} width="fit-content">
        Import
      </Button>

      <Modal isOpen={isOpen} onClose={onClose} size="xl">
        <ModalOverlay />
        <ModalContent>
          <ModalHeader>Import PikPak Folder</ModalHeader>
          <ModalCloseButton />
          <ModalBody>
            <VStack spacing="2" alignItems="start">
              <FormControl isInvalid={!!formErrs.bangumi}>
                <FormLabel>Bangumi ID</FormLabel>
                <Input
                  {...register("bangumi", {
                    required: true,
                    validate: (value) => !Number.isNaN(Number.parseInt(value)),
                  })}
                />
                <FormErrorMessage>{formErrs.bangumi?.message}</FormErrorMessage>
              </FormControl>
              <List spacing={3} width="100%">
                {bangumiList.map(([name, id]) => (
                  <ListItem
                    key={id}
                    onClick={() => setValue("bangumi", id.toString(), { shouldValidate: true, shouldDirty: true })}
                    display="flex"
                    alignItems="center"
                  >
                    <ListIcon as={IoFilmOutline} color="green.500" marginInlineStart="2" />
                    {name}
                  </ListItem>
                ))}
              </List>
              <FormControl isInvalid={!!formErrs.path}>
                <FormLabel>Target Folder</FormLabel>
                <Input {...register("path", { required: true })} />
                <FormErrorMessage>{formErrs.path?.message}</FormErrorMessage>
              </FormControl>
              <List spacing={3} width="100%" height="48" overflowY="auto" border="gray" borderWidth="1">
                {fileList.map((name) => (
                  <ListItem key={id} display="flex" alignItems="center">
                    <HStack width="100%">
                      <ListIcon as={IoDocumentOutline} color="green.500" marginInlineStart="2" />
                      <Text flexGrow="1">{name}</Text>
                      <IconButton
                        icon={<Icon as={IoCheckmarkOutline} />}
                        aria-label="select"
                        onClick={() => setValue("path", name, { shouldValidate: true, shouldDirty: true })}
                      />
                    </HStack>
                  </ListItem>
                ))}
              </List>
              <FormControl isInvalid={!!formErrs.regex}>
                <FormLabel>RegExp</FormLabel>
                <InputGroup>
                  <Textarea {...register("regex", { required: true })} />
                  <InputRightElement>
                    <IconButton
                      icon={<IoCodeWorkingOutline />}
                      aria-label="Escape regex"
                      onClick={onEscapeRegex}
                      size="sm"
                    />
                  </InputRightElement>
                </InputGroup>
                <FormErrorMessage>{formErrs.regex?.message}</FormErrorMessage>
              </FormControl>
              <FormControl isInvalid={!!formErrs.match_group_ep}>
                <FormLabel>Episode ID Group ID</FormLabel>
                <Input type="number" {...register("match_group_ep", { required: true, valueAsNumber: true })} />
                <FormErrorMessage>{formErrs.match_group_ep?.message}</FormErrorMessage>
              </FormControl>
            </VStack>
          </ModalBody>

          <ModalFooter>
            <Button onClick={onClose} mr="2">
              Close
            </Button>
            <Button
              colorScheme="teal"
              // eslint-disable-next-line @typescript-eslint/no-misused-promises
              onClick={handleSubmit(onSubmit)}
              isDisabled={!isDirty || isValidating || !isValid}
            >
              Save
            </Button>
          </ModalFooter>
        </ModalContent>
      </Modal>
    </>
  );
};
