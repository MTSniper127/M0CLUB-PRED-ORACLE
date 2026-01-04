
FROM node:20-slim as builder
WORKDIR /app
COPY package.json tsconfig.json next.config.js /app/
COPY src /app/src
COPY public /app/public
RUN npm install
RUN npm run build

FROM node:20-slim
WORKDIR /app
ENV NODE_ENV=production
COPY --from=builder /app/.next/standalone /app/
COPY --from=builder /app/.next/static /app/.next/static
COPY --from=builder /app/public /app/public
EXPOSE 3000
CMD ["node", "server.js"]
