FROM --platform=$BUILDPLATFORM mcr.microsoft.com/dotnet/sdk:8.0 AS build

ARG TARGETARCH

RUN apt update && \
    apt install -y ca-certificates curl gnupg2 && \
    mkdir -p /etc/apt/keyrings && \
    curl -fsSL https://deb.nodesource.com/gpgkey/nodesource-repo.gpg.key \
        | gpg --dearmor -o /etc/apt/keyrings/nodesource.gpg && \
    echo "deb [signed-by=/etc/apt/keyrings/nodesource.gpg] https://deb.nodesource.com/node_20.x nodistro main" \
        | tee /etc/apt/sources.list.d/nodesource.list && \
    apt install -y build-essential nodejs npm && \
    npm install -g pnpm

COPY "Net.Vatprc.Uniapi.sln" "/src/"
COPY "Net.Vatprc.Uniapi/Net.Vatprc.Uniapi.csproj" "/src/Net.Vatprc.Uniapi/"
COPY "Net.Vatprc.Uniapi/packages.lock.json" "/src/Net.Vatprc.Uniapi/"
COPY "Net.Vatprc.Uniapi.UI/package.json" "/src/Net.Vatprc.Uniapi.UI/"
COPY "Net.Vatprc.Uniapi.UI/pnpm-lock.yaml" "/src/Net.Vatprc.Uniapi.UI/"
WORKDIR "/src"
RUN dotnet restore "Net.Vatprc.Uniapi" --locked-mode -a $TARGETARCH

COPY "." "/src/"
RUN dotnet build "Net.Vatprc.Uniapi" -c Release --no-self-contained --no-restore -a $TARGETARCH
RUN dotnet publish "Net.Vatprc.Uniapi" -c Release -o /app/publish --no-self-contained --no-restore -a $TARGETARCH

FROM mcr.microsoft.com/dotnet/aspnet:8.0 AS final
WORKDIR /app
EXPOSE 5000
COPY --from=build /app/publish .
ENTRYPOINT ["dotnet", "Net.Vatprc.Uniapi.dll"]
