package pl.superhero.handlers;

import io.vertx.core.Handler;
import io.vertx.core.json.JsonObject;
import io.vertx.ext.mongo.MongoClient;
import io.vertx.ext.web.api.RequestParameters;
import io.vertx.ext.web.RoutingContext;

public class ShowHeroByIdHandler implements Handler<RoutingContext> {

  private final MongoClient mongo;

  public ShowHeroByIdHandler(MongoClient mongo) {
    this.mongo = mongo;
  }

  @Override
  public void handle(RoutingContext routingContext) {
    RequestParameters params = routingContext.get("parsedParameters");
    String heroId = params.pathParameter("heroId").getString();
    JsonObject query = new JsonObject().put("_id", heroId);
    mongo.findOne("heroes", query, null, r -> {
      if (r.succeeded()) {
        if (r.result() == null) {
          routingContext.response().setStatusCode(404).end();
        } else {
          routingContext.response().end(r.result().toBuffer());
        }
      } else {
        routingContext.fail(r.cause());
      }
    });
  }

}
