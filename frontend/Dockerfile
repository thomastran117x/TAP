FROM node:24-alpine AS dependencies
WORKDIR /app
COPY package*.json ./
RUN npm ci

FROM dependencies AS build
COPY . .
RUN npm run build

FROM node:24-alpine AS production
WORKDIR /app
ENV NODE_ENV=production
COPY --from=build /app/dist/frontend ./dist/frontend
EXPOSE 4000
CMD ["node", "dist/frontend/server/server.mjs"]
