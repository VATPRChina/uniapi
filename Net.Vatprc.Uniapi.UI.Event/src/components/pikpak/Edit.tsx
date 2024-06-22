import api, { ApiError } from "../../api";
import { promiseWithLog, toast } from "../../utils";
import { NullULID } from "../utils/misc";
import {
  Button,
  FormControl,
  FormErrorMessage,
  FormLabel,
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
  Switch,
  Textarea,
  VStack,
  useDisclosure,
} from "@chakra-ui/react";
import escape from "escape-string-regexp";
import React, { useEffect, useState } from "react";
import { SubmitHandler, useForm } from "react-hook-form";
import { IoCodeWorkingOutline, IoCreateOutline, IoFilmOutline } from "react-icons/io5";
import { useDebounce } from "usehooks-ts";

type Inputs = {
  bangumi: string;
  target: string;
  regex: string;
  match_group_ep: number;
  enabled: boolean;
};

export const Edit: React.FunctionComponent<{ id?: string }> = ({ id }) => {
  const { isOpen, onOpen, onClose } = useDisclosure();
  const {
    register,
    handleSubmit,
    formState: { errors: formErrs, isDirty, isValid, isValidating },
    setValue,
    getValues,
    watch,
  } = useForm<Inputs>({
    defaultValues: async (): Promise<Inputs> => {
      if (!id) return { enabled: true, bangumi: "", target: "", regex: "", match_group_ep: 1 };
      const res = await api.pikPakGet(id ?? NullULID);
      if (res.error) console.error(res.error);
      const job = res.data;
      return {
        bangumi: job?.bangumi.toString(),
        target: job?.target,
        regex: job?.regex,
        match_group_ep: job?.match_group_ep,
        enabled: job?.enabled,
      };
    },
  });
  const onSubmit: SubmitHandler<Inputs> = async (inputs) => {
    try {
      if (id) {
        await api.pikPakUpdate(id, {
          bangumi: Number.parseInt(inputs.bangumi),
          target: inputs.target,
          regex: inputs.regex,
          match_group_ep: inputs.match_group_ep,
          enabled: inputs.enabled,
        });
        await api.mutatePikPakGet(id);
      } else {
        await api.pikPakCreate({
          bangumi: Number.parseInt(inputs.bangumi),
          target: inputs.target,
          regex: inputs.regex,
          match_group_ep: inputs.match_group_ep,
        });
      }
      await api.mutatePikPakList();
      toast({
        title: "Job created.",
        status: "success",
        duration: 5000,
        isClosable: true,
      });
      onClose();
    } catch (err) {
      toast({
        title: (err as Error | ApiError).message,
        status: "error",
        duration: 5000,
        isClosable: true,
      });
    }
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

  return (
    <>
      {id ? (
        <IconButton aria-label="Edit" icon={<Icon as={IoCreateOutline} />} onClick={onOpen} />
      ) : (
        <Button colorScheme="teal" onClick={onOpen} width="fit-content">
          Create
        </Button>
      )}

      <Modal isOpen={isOpen} onClose={onClose} size="xl">
        <ModalOverlay />
        <ModalContent>
          <ModalHeader>Edit PikPak Task</ModalHeader>
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
              <FormControl isInvalid={!!formErrs.target}>
                <FormLabel>Target Folder</FormLabel>
                <Input {...register("target", { required: true })} />
                <FormErrorMessage>{formErrs.target?.message}</FormErrorMessage>
              </FormControl>
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
              <FormControl display="flex" alignItems="center">
                <FormLabel mb={0}>Enabled</FormLabel>
                <Switch {...register("enabled", { disabled: !id })} />
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
