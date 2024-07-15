import { ANONYMOUS_CID, devLogin, useUser } from "@/services/auth";
import { promiseWithToast } from "@/utils";
import { Button, Popover, TextInput, Title } from "@mantine/core";
import { useForm } from "@tanstack/react-form";

export const DevLogin = () => {
  const user = useUser();

  const form = useForm({
    defaultValues: { username: "" },
    onSubmit: async ({ value }) => {
      await devLogin(value.username, "foobar");
    },
  });

  if (import.meta.env.PROD) return null;
  if (user.cid !== ANONYMOUS_CID) return null;
  return (
    <Popover>
      <Popover.Target>
        <Button variant="outline">Dev Login</Button>
      </Popover.Target>
      <Popover.Dropdown>
        <form
          onSubmit={(e) => {
            e.preventDefault();
            e.stopPropagation();
            promiseWithToast(form.handleSubmit());
          }}
        >
          <Title order={4}>Login</Title>
          <form.Field
            name="username"
            children={(field) => (
              <TextInput label="Username" onChange={(e) => field.handleChange(e.target.value)}></TextInput>
            )}
          />
          <Button variant="outline" type="submit">
            Login
          </Button>
        </form>
      </Popover.Dropdown>
    </Popover>
  );
};
