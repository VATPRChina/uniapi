FROM --platform=$BUILDPLATFORM mcr.microsoft.com/dotnet/sdk:9.0 AS build

ARG TARGETARCH
ARG VERSION=0.0.1

RUN apt update && \
    apt install -y ca-certificates curl gnupg2

RUN curl -sL https://sentry.io/get-cli/ | sh

COPY "Net.Vatprc.Uniapi.csproj" "/src/"
COPY "packages.lock.json" "/src/"
WORKDIR "/src/"
RUN dotnet restore --locked-mode -a $TARGETARCH

COPY "." "/src/"
RUN sed -i 's|<AssemblyVersion>0.0.1</AssemblyVersion>|<AssemblyVersion>${VERSION}</AssemblyVersion>|g' "/src/Net.Vatprc.Uniapi.csproj"
RUN --mount=type=secret,id=SENTRY_AUTH_TOKEN,env=SENTRY_AUTH_TOKEN \
    dotnet build -c Release --no-self-contained --no-restore -a $TARGETARCH
RUN --mount=type=secret,id=SENTRY_AUTH_TOKEN,env=SENTRY_AUTH_TOKEN \
    dotnet publish -c Release -o /app/publish --no-self-contained --no-restore -a $TARGETARCH

FROM mcr.microsoft.com/dotnet/aspnet:9.0 AS final
WORKDIR /app
EXPOSE 5000
COPY --from=build /app/publish .
ENTRYPOINT ["dotnet", "Net.Vatprc.Uniapi.dll"]
