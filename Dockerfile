FROM gradle:jdk17-alpine AS build
COPY --chown=gradle:gradle . /home/gradle/src
WORKDIR /home/gradle/src
RUN gradle fatJar --no-daemon

FROM openjdk:17-alpine

EXPOSE 4567

RUN mkdir /app

COPY --from=build /home/gradle/src/build/libs/*.jar /app/application.jar

ENTRYPOINT ["java", "-jar","/app/application.jar"]
