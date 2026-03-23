FROM node:20-alpine

WORKDIR /app

COPY package*.json ./

RUN npm ci --omit=dev && npm cache clean --force

COPY . .

RUN npm run build

EXPOSE 8080

CMD ["npm", "start"]
