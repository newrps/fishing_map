# ----- builder stage -----
FROM node:22-alpine AS builder
WORKDIR /app

COPY package*.json ./
RUN npm ci

COPY . .

# VITE_VWORLD_KEY must be passed as build arg (Vite inlines it)
ARG VITE_VWORLD_KEY
ENV VITE_VWORLD_KEY=$VITE_VWORLD_KEY

RUN npm run build

# ----- runtime stage -----
FROM nginx:1.27-alpine

COPY --from=builder /app/build /usr/share/nginx/html
COPY nginx.conf /etc/nginx/conf.d/default.conf

EXPOSE 80
CMD ["nginx", "-g", "daemon off;"]
