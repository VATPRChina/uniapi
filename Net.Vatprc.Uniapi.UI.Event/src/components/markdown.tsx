import { TypographyStylesProvider } from "@mantine/core";
import dompurify from "dompurify";
import { parse } from "marked";

export const Markdown = ({ children }: { children?: string }) => {
  if (!children) return null;

  const html = dompurify.sanitize(parse(children, { async: false, gfm: true, breaks: true }) as string);

  return (
    <TypographyStylesProvider>
      <div dangerouslySetInnerHTML={{ __html: html }} />
    </TypographyStylesProvider>
  );
};
