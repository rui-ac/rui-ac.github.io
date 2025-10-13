FROM ruby:3.4-slim-bookworm AS builder

RUN apt-get update -qq && apt-get install -y build-essential

WORKDIR /app

COPY Gemfile Gemfile.lock ./

RUN bundle install --jobs 4 --retry 3

COPY . .

RUN bundle exec jekyll build -d /app/_site

FROM nginx:1.29-alpine-slim

COPY --from=builder /app/_site /usr/share/nginx/html

EXPOSE 80

ENTRYPOINT ["nginx", "-g", "daemon off;"]
