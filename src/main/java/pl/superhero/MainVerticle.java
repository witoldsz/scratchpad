package pl.superhero;

import io.vertx.core.AbstractVerticle;
import io.vertx.core.http.HttpServer;
import io.vertx.core.http.HttpServerOptions;
import io.vertx.ext.web.api.contract.openapi3.OpenAPI3RouterFactory;
import io.vertx.ext.web.Router;
import io.vertx.core.Future;
import static io.vertx.core.Future.future;
import io.vertx.core.json.JsonArray;
import io.vertx.core.json.JsonObject;
import io.vertx.ext.mongo.MongoClient;
import java.util.List;
import static java.util.Optional.ofNullable;

public class MainVerticle extends AbstractVerticle {

  HttpServer server;
  MongoClient mongo;

  @Override
  public void start(Future done) {
    System.setProperty("org.mongodb.async.type", "netty");
    mongo = MongoClient.createShared(vertx, new JsonObject()
        .put("connection_string",
            "mongodb://witoldsz:Q0PV6aCrggdTz1Kv"
            + "@superhero-shard-00-00-ty8ms.mongodb.net:27017,"
            + "superhero-shard-00-01-ty8ms.mongodb.net:27017,"
            + "superhero-shard-00-02-ty8ms.mongodb.net:27017"
            + "/testing"
            + "?ssl=true"
            + "&replicaSet=superhero-shard-0"
            + "&authSource=admin"
            + "&retryWrites=true"
        )
    );

    Future<List<String>> mongoF = future();
    mongo.getCollections(mongoF.completer());

    mongoF
        .compose(collections -> {
          System.out.println("Collections: " + collections);
          Future<OpenAPI3RouterFactory> routerFactoryF = future();
          OpenAPI3RouterFactory.createRouterFactoryFromFile(
              this.vertx,
              getClass().getResource("/superheroes.json").getFile(),
              r -> {
                System.out.println("r!!!!!!!!!!!!!!!!!!!!!!!!!!!!!1 = " + r);
              }
          );
          return routerFactoryF;
        })
        .compose(routerFactory -> {

        }, done);
//        .compose(routerFactory -> {
//          System.out.println("routerFactory: " + routerFactory);
//          // Enable automatic response when ValidationException is thrown
//          routerFactory.enableValidationFailureHandler(true);
//
//          // Add routes handlers
//          routerFactory.addHandlerByOperationId("listHeroes", new pl.superhero.handlers.ListHeroesHandler());
//          routerFactory.addHandlerByOperationId("createHero", new pl.superhero.handlers.CreateHeroHandler());
//          routerFactory.addHandlerByOperationId("showHeroById", new pl.superhero.handlers.ShowHeroByIdHandler());
//
//          // Generate the router
//          Router router = routerFactory.getRouter();
//          server = vertx.createHttpServer(new HttpServerOptions().setPort(8080).setHost("localhost"));
//          server.requestHandler(router::accept).listen();
//          done.complete();
//        }, done);
  }

  @Override
  public void stop() {
    ofNullable(server).ifPresent(HttpServer::close);
    ofNullable(mongo).ifPresent(MongoClient::close);
  }

}
