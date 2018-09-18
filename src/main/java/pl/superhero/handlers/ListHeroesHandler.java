package pl.superhero.handlers;

import io.vertx.core.Handler;
import io.vertx.ext.web.api.RequestParameters;
import io.vertx.ext.web.RoutingContext;

public class ListHeroesHandler implements Handler<RoutingContext> {

    public ListHeroesHandler(){

    }

    @Override
    public void handle(RoutingContext routingContext) {
        RequestParameters params = routingContext.get("parsedParameters");
        // Handle listHeroes
        routingContext.response().setStatusCode(501).setStatusMessage("Not Implemented").end();
    }

}