FROM --platform=$BUILDPLATFORM mcr.microsoft.com/dotnet/sdk:8.0 AS build

ARG TARGETARCH

RUN apt update && \
    apt install -y ca-certificates curl gnupg2

COPY "Net.Vatprc.Uniapi.csproj" "/src/"
COPY "packages.lock.json" "/src/"
WORKDIR "/src/"
RUN dotnet restore --locked-mode -a $TARGETARCH

COPY "." "/src/"
RUN dotnet build -c Release --no-self-contained --no-restore -a $TARGETARCH
RUN dotnet publish -c Release -o /app/publish --no-self-contained --no-restore -a $TARGETARCH

FROM mcr.microsoft.com/dotnet/aspnet:8.0 AS final
WORKDIR /app
EXPOSE 5000
COPY --from=build /app/publish .
ENTRYPOINT ["dotnet", "Net.Vatprc.Uniapi.dll"]
