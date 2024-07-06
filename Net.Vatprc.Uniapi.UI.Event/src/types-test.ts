/* eslint-disable @typescript-eslint/ban-types */
import { paths } from "./api";
import { Any, Object, Test } from "ts-toolbelt";

Test.check<Any.Equals<{ query?: never }, { query: never }>, 1, Test.Pass>();
Test.check<{ query: { path: string } }, 1, Test.Pass>();

type OptionalParameters<Parameters, Key extends string> = Parameters extends { [K in Key]?: infer T }
  ? { [K in Key]?: Partial<T> }
  : never;

Test.checks([
  Test.check<
    OptionalParameters<paths["/api/events/{eid}/slots/{sid}/booking"]["get"]["parameters"], "query">,
    { query?: never },
    Test.Pass
  >(),
  Test.check<
    OptionalParameters<paths["/api/events/{eid}/slots/{sid}/booking"]["get"]["parameters"], "path">,
    { path?: { eid?: string; sid?: string } },
    Test.Pass
  >(),
]);

Test.checks([
  Test.check<
    Object.Exclude<paths["/api/events/{eid}/slots/{sid}/booking"]["get"]["parameters"]["path"], { sid: string }>,
    { eid: string },
    Test.Pass
  >(),
  Test.check<
    Object.Exclude<
      paths["/api/events/{eid}/slots/{sid}/booking"]["get"]["parameters"]["path"],
      Record<string, never>,
      "equals"
    >,
    { sid: string; eid: string },
    Test.Pass
  >(),
  Test.check<
    Object.Exclude<
      paths["/api/events/{eid}/slots/{sid}/booking"]["get"]["parameters"]["path"],
      Record<string, never>,
      "equals"
    >,
    { eid: string },
    Test.Fail
  >(),
]);

type ParameterOf<Parameters, Key extends string> = Parameters extends { [K in Key]?: infer T extends object }
  ? T
  : never;
type RestParameters<Parameters, Key extends string, Existing extends OptionalParameters<Parameters, Key>> = {
  [K in Key]: Object.Exclude<ParameterOf<Parameters, Key>, ParameterOf<Existing, Key>>;
};

Test.checks([
  Test.check<
    ParameterOf<paths["/api/events/{eid}/slots/{sid}/booking"]["get"]["parameters"], "path">,
    { eid: string; sid: string },
    Test.Pass
  >(),
  Test.check<ParameterOf<{ path: { eid: string } }, "path">, { eid: string }, Test.Pass>(),
  Test.check<ParameterOf<{ path: Record<string, never> }, "path">, Record<string, never>, Test.Pass>(),

  Test.check<
    keyof Object.Exclude<{ eid: string; sid: string }, Record<string, never>>,
    // { eid: string; sid: string },
    never,
    Test.Pass
  >(),

  Test.check<keyof {}, never, Test.Pass>(),

  Test.check<
    RestParameters<
      paths["/api/events/{eid}/slots/{sid}/booking"]["get"]["parameters"],
      "path",
      { path: { eid: string } }
    >,
    { path: { sid: string } },
    Test.Pass
  >(),

  Test.check<
    RestParameters<paths["/api/events/{eid}/slots/{sid}/booking"]["get"]["parameters"], "path", Record<string, never>>,
    { path: { eid: string; sid: string } },
    Test.Pass
  >(),
]);

type OmitKeyIfEmpty<T, K extends string> = T extends { [Key in K]: infer V }
  ? Any.Equals<keyof V, never> extends 1
    ? {}
    : { [Key in K]: V }
  : {};

Test.checks([
  Test.check<OmitKeyIfEmpty<{ path: { eid: string } }, "path">, { path: { eid: string } }, Test.Pass>(),
  Test.check<OmitKeyIfEmpty<{ path: {} }, "path">, {}, Test.Pass>(),
  Test.check<OmitKeyIfEmpty<{ query: { a: 1 } }, "path">, {}, Test.Pass>(),
]);
