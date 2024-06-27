import { Button } from "./ui/button";
import { Input } from "./ui/input";
import { devLogin, useUser } from "@/services/auth";
import { Label } from "@radix-ui/react-label";
import { Popover, PopoverContent, PopoverTrigger } from "@radix-ui/react-popover";
import { useForm } from "@tanstack/react-form";

export const DevLogin = () => {
  const user = useUser();

  const form = useForm({
    defaultValues: {
      username: "",
    },
    onSubmit: async ({ value }) => {
      // Do something with form data
      await devLogin(value.username, "foobar");
    },
  });

  if (user) return null;
  return (
    <Popover>
      <PopoverTrigger asChild>
        <Button variant="outline">Login</Button>
      </PopoverTrigger>
      <PopoverContent className="w-80">
        <form
          className="grid gap-4"
          onSubmit={(e) => {
            e.preventDefault();
            e.stopPropagation();
            // eslint-disable-next-line no-console
            form.handleSubmit().catch(console.error);
          }}
        >
          <div className="space-y-2">
            <h4 className="font-medium leading-none">Login</h4>
          </div>
          <div className="grid gap-2">
            <div className="grid grid-cols-3 items-center gap-4">
              <Label htmlFor="username">Username</Label>
              <form.Field
                name="username"
                // eslint-disable-next-line react/no-children-prop
                children={(field) => (
                  <Input
                    id="username"
                    className="col-span-2 h-8"
                    onChange={(e) => field.handleChange(e.target.value)}
                  />
                )}
              />
            </div>
          </div>
          <Button variant="outline">Login</Button>
        </form>
      </PopoverContent>
    </Popover>
  );
};
