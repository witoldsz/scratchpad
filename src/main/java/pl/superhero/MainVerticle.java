package pl.superhero;

import io.vertx.core.AbstractVerticle;
import io.vertx.core.http.HttpServer;
import io.vertx.core.http.HttpServerOptions;
import io.vertx.ext.web.api.contract.openapi3.OpenAPI3RouterFactory;
import io.vertx.ext.web.Router;
import io.vertx.core.Future;
import static io.vertx.core.Future.future;
import io.vertx.core.json.JsonObject;
import io.vertx.ext.mongo.MongoClient;
import io.vertx.ext.web.api.contract.RouterFactoryOptions;
import io.vertx.ext.web.handler.StaticHandler;
import java.util.List;
import pl.superhero.handlers.CreateHeroHandler;
import pl.superhero.handlers.ListHeroesHandler;
import pl.superhero.handlers.ShowHeroByIdHandler;
import static java.util.Optional.ofNullable;

public class MainVerticle extends AbstractVerticle {

  private HttpServer server;
  private MongoClient mongo;

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

    // Just to make sure the connection works
    Future<List<String>> mongoF = future();
    mongo.getCollections(mongoF.completer());

    mongoF
      .compose(__ -> createRouterFactory())
      .compose(routerFactory -> {
        Router router = configureRouter(routerFactory);
        server = vertx.createHttpServer(new HttpServerOptions().setPort(8080).setHost("localhost"));
        server.requestHandler(router::accept).listen();
        done.complete();
      }, done);
  }

  @Override
  public void stop() {
    ofNullable(server).ifPresent(HttpServer::close);
    ofNullable(mongo).ifPresent(MongoClient::close);
  }

  private Future<OpenAPI3RouterFactory> createRouterFactory() {
    Future<OpenAPI3RouterFactory> routerFactoryF = future();
    OpenAPI3RouterFactory.create(this.vertx, "webroot/superheroes.yaml", routerFactoryF.completer());
    return routerFactoryF;
  }

  private Router configureRouter(OpenAPI3RouterFactory routerFactory) {
    routerFactory.setOptions(new RouterFactoryOptions()
      .setMountNotImplementedHandler(true)
      .setMountValidationFailureHandler(true));

    // Add routes handlers
    routerFactory.addHandlerByOperationId("listHeroes", new ListHeroesHandler(mongo));
    routerFactory.addHandlerByOperationId("createHero", new CreateHeroHandler(mongo));
    routerFactory.addHandlerByOperationId("showHeroById", new ShowHeroByIdHandler(mongo));

    Router router = routerFactory.getRouter();

    int oneMinute = 60;
    router.route("/*").handler(StaticHandler.create().setMaxAgeSeconds(oneMinute));
    return router;

  }

}
